/*
 * This file is part of the Tarnish package.
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

#![feature(phase)]

extern crate regex;
#[phase(plugin, link)] extern crate regex_macros;

use std::fmt::Show;
use regex::Regex;
pub use Either::{Left, Right};
pub use ParserErrorKind::{InvalidParameter, UnexpectedInput};

pub type ParserResult<T> = Result<T, ParserError>;

pub trait Parser<U> {
    fn parse<T: Str>(&mut self, input: T) -> ParserResult<(U, uint)>;
}

pub struct ParserError {
    kind: ParserErrorKind,
    msg: String
}

pub struct Lit {
    regexp: Regex
}

pub struct Apply<'a, T, U: Parser<T>, V> {
    parser: U,
    cb: |T|: 'a -> V
}

pub struct And<T, U, V: Parser<T>, W: Parser<U>> {
    a: V,
    b: W
}

pub struct Or<T, U, V: Parser<T>, W: Parser<U>> {
    a: V,
    b: W
}

pub struct Seq2<T, U> {
    pub first: T,
    pub second: U
}

pub enum ParserErrorKind {
    InvalidParameter,
    UnexpectedInput
}

pub enum Either<T, U> {
    Left(T),
    Right(U)
}

impl Parser<String> for Lit {
    fn parse<T: Str>(&mut self, input: T) -> ParserResult<(String, uint)> {
        match self.regexp.captures(input.as_slice()) {
            Some(cap) => {
                let res = cap.at(0).to_string();
                let len = res.len();
                Ok((res, len))
            }
            None => Err(ParserError { kind: UnexpectedInput, msg: "literal did not match".to_string() })
        }
    }
}

impl<'a, T, U: Parser<T>, V> Parser<V> for Apply<'a, T, U, V> {
    fn parse<W: Str>(&mut self, input: W) -> ParserResult<(V, uint)> {
        let (output, adv) = try!(self.parser.parse(input));
        Ok(((self.cb)(output), adv))
    }
}

impl<T, U, V: Parser<T>, W: Parser<U>> Parser<Seq2<T, U>> for And<T, U, V, W> {
    fn parse<Z: Str>(&mut self, input: Z) -> ParserResult<(Seq2<T, U>, uint)> {
        let slice = input.as_slice();
        let (fres, fadv) = try!(self.a.parse(slice));
        let (sres, sadv) = try!(self.b.parse(slice.slice_from(fadv)));
        Ok((Seq2 { first: fres, second: sres }, fadv + sadv))
    }
}

impl<T, U, V: Parser<T>, W: Parser<U>> Parser<Either<T, U>> for Or<T, U, V, W> {
    fn parse<Z: Str>(&mut self, input: Z) -> ParserResult<(Either<T, U>, uint)> {
        let slice = input.as_slice();
        match self.a.parse(slice) {
            Ok((res, adv)) => Ok((Left(res), adv)),
            _ => {
                let (res, adv) = try!(self.b.parse(slice));
                Ok((Right(res), adv))
            }
        }
    }
}

#[inline]
pub fn lit<T: Str + Show>(regex_str: T) -> Lit {
    match Regex::new((format!(r"\A{}", regex_str)).as_slice()) {
        Ok(regexp) => Lit { regexp: regexp },
        Err(err) => panic!("{}", err)
    }
}

#[inline]
pub fn apply<'a, T, U: Parser<T>, V>(parser: U, cb: |T|: 'a -> V) -> Apply<'a, T, U, V> {
    Apply { parser: parser, cb: cb }
}

#[inline]
pub fn and<T, U, V: Parser<T>, W: Parser<U>>(a: V, b: W) -> And<T, U, V, W> {
    And { a: a, b: b }
}

#[inline]
pub fn or<T, U, V: Parser<T>, W: Parser<U>>(a: V, b: W) -> Or<T, U, V, W> {
    Or { a: a, b: b }
}
