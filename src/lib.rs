/*
 * This file is part of the Tarnish package.
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

#![feature(if_let, macro_rules, phase, unboxed_closures)]

extern crate regex;
#[phase(plugin, link)] extern crate regex_macros;

use std::fmt::Show;
use regex::Regex;
pub use Either::{Left, Right};
pub use ParserErrorKind::{InvalidParameter, UnexpectedInput};

#[macro_export]
macro_rules! lit (
    ($regexp:expr) => (::tarnish::lit($regexp))
)

#[macro_export]
macro_rules! apply (
    ($parser:expr, $closure:expr) => (::tarnish::apply(&*$parser, $closure))
)

#[macro_export]
macro_rules! and (
    ($a:expr, $b:expr) => (::tarnish::and(&*$a, &*$b))
)

#[macro_export]
macro_rules! or (
    ($a:expr, $b:expr) => (::tarnish::or(&*$a, &*$b))
)

#[macro_export]
macro_rules! until (
    ($parser:expr, $delim:expr) => (::tarnish::until(&*$parser, $delim))
)

#[macro_export]
macro_rules! slurp (
    ($parser:expr) => (::tarnish::slurp(&*$parser))
)

#[macro_export]
macro_rules! concat (
    ($parser:expr) => (::tarnish::concat(&*$parser))
)

pub type ParserResult<T> = Result<T, ParserError>;

pub type StackParser<'a, T> = Fn(&mut State<'a>) -> ParserResult<T> + 'a;
pub type Parser<'a, T> = Box<StackParser<'a, T>>;

pub struct State<'a> {
    text: &'a str,
    index: uint
}

pub struct Seq2<T, U> {
    pub first: T,
    pub second: U
}

pub struct ParserError {
    pub kind: ParserErrorKind,
    pub msg: String
}

pub enum ParserErrorKind {
    InvalidParameter,
    UnexpectedInput
}

pub enum Either<T, U> {
    Left(T),
    Right(U)
}

impl<'a> State<'a> {
    pub fn new(text: &'a str) -> State<'a> {
        State {
            text: text,
            index: 0
        }
    }

    pub fn reset(&mut self, text: &'a str) {
        self.text = text;
        self.index = 0;
    }
}

#[inline]
pub fn lit<'a>(regex_str: &'a str) -> Parser<'a, String> {
    box move |&: input| {
        match Regex::new((format!(r"\A{}", regex_str)).as_slice()) {
            Ok(regexp) => match regexp.captures(input.text.slice_from(input.index)) {
                Some(cap) => {
                    let res = cap.at(0).to_string();
                    input.index += res.len();
                    Ok(res)
                }
                None => Err(ParserError { kind: UnexpectedInput, msg: "literal did not match".to_string() })
            },
            Err(err) => panic!("{}", err)
        }
    }
}

#[inline]
pub fn apply<'a, T, U, V: Fn(T) -> U + 'a>(parser: &'a StackParser<'a, T>, cb: V) -> Parser<'a, U> {
    box move |&: input| {
        let output = try!(parser.call((input,)));
        Ok(cb(output))
    }
}

#[inline]
pub fn and<'a, T, U>(a: &'a StackParser<'a, T>, b: &'a StackParser<'a, U>) -> Parser<'a, Seq2<T, U>> {
    box move |&: input| {
        let fres = try!(a.call((input,)));
        let sres = try!(b.call((input,)));
        Ok(Seq2 { first: fres, second: sres })
    }
}

#[inline]
pub fn or<'a, T, U>(a: &'a StackParser<'a, T>, b: &'a StackParser<'a, U>) -> Parser<'a, Either<T, U>> {
    box move |&: input| {
        match a.call((input,)) {
            Ok(res) => Ok(Left(res)),
            _ => {
                let res = try!(b.call((input,)));
                Ok(Right(res))
            }
        }
    }
}

#[inline]
pub fn until<'a, T: PartialEq>(parser: &'a StackParser<'a, T>, delim: Option<&'a StackParser<'a, T>>) -> Parser<'a, Vec<T>> {
    box move |&: input| {
        let mut output = vec!();
        while input.index < input.text.len() {
            let val = try!(parser.call((input,)));
            if let Some(delim) = delim {
                if let Ok(dval) = delim.call((input,)) {
                    if val == dval {
                        break;
                    }
                }
            }
            output.push(val);
        }
        Ok(output)
    }
}

#[inline]
pub fn slurp<'a, T>(parser: &'a StackParser<'a, T>) -> Parser<'a, Vec<T>> {
    box move |&: input| {
        let mut output = vec!();
        while input.index < input.text.len() {{
            match parser.call((input,)) {
                Ok(val) => output.push(val),
                Err(err) => {
                    if output.len() == 0 {
                        return Err(err);
                    } else {
                        break
                    }
                }
            }
        }}
        Ok(output)
    }
}

#[inline]
pub fn concat<'a, T: Add<T, T> + Copy>(parser: &'a StackParser<'a, Vec<T>>) -> Parser<'a, T> {
    box move |&: input| {
        let vec = try!(parser.call((input,)));
        if vec.len() == 0 {
            Err(ParserError { kind: UnexpectedInput, msg: "Invalid vector length in concat (length was 0)".to_string() })
        } else {
            let mut output = vec[0];
            for val in vec.slice_from(1).iter() {
                output = output + *val;
            }
            Ok(output)
        }
    }
}

