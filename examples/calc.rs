#![feature(phase, unboxed_closures)]

#[phase(plugin, link)]
extern crate tarnish;

use std::io;
use tarnish::{State, Left, Right};

fn main() {
    let numstr = lit!(r"[0-9]+(?:\.[0-9]+)?");
    let plus = lit!(r"\+");
    let minus = lit!(r"-");

    let num = apply!(numstr, |&: numstr: String| from_str::<f64>(numstr.as_slice()).unwrap());
    
    let add_inner = and!(plus, num);
    let add_num = apply!(add_inner, |&: seq| seq.second);

    let sub_inner = and!(minus, num);
    let sub_num = apply!(sub_inner, |&: seq| -seq.second);
    
    let slurp_add = slurp!(add_num);
    let slurp_sub = slurp!(sub_num);
    let or_slurp = or!(slurp_add, slurp_sub);
    let pick_slurp = apply!(or_slurp, |&: either| {
        match either {
            Left(lval) => lval,
            Right(rval) => rval
        }
    });
    let concat_slurp = concat!(pick_slurp);
    let slurp_concat = slurp!(concat_slurp);
    let concat_concat = concat!(slurp_concat);
    let and_num_slurp = and!(num, concat_concat);
    let expr = apply!(and_num_slurp, |&: seq| seq.first + seq.second);

    let parser = expr;
    let mut state = State::new("");
    for line in io::stdin().lines() {
        let line = line.unwrap();
        state.reset(line.as_slice());
        let num = match parser.call((&mut state,)) {
            Ok(num) => num,
            Err(err) => panic!("{}", err.msg)
        };
        println!("{}", num);
    }
}
