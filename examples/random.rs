#![feature(phase)]

#[phase(plugin, link)]
extern crate tarnish;

use tarnish::State;

fn main() {
    let ident = lit!("[a-zA-Z_][a-zA-Z0-9_]*");
    let number = lit!("[0-9]+");
    let numparser = apply!(number, |numstr| from_str::<uint>(numstr.as_slice()).unwrap());
    let num = numparser.call((&mut State::new("24"),)).ok().unwrap();
    println!("{}", num + 5);
}
