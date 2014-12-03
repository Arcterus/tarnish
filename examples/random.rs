extern crate tarnish;

use tarnish::{Parser, lit, apply, and, or};

fn main() {
    let ident = lit("[a-zA-Z_][a-zA-Z0-9_]*");
    let number = lit("[0-9]+");
    let mut numparser = apply(number, |numstr| from_str::<uint>(numstr.as_slice()).unwrap());
    let (num, _) = numparser.parse("24").ok().unwrap();
    println!("{}", num + 5);
}
