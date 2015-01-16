#![allow(unstable)]

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;

fn main() {
    let x: Vec<usize> = vec![4,2,1,6,3,1,2,7,4,2];
    let parser = parsing::Parser::new(x.as_slice());
    let thought = parser.next();
    println!("{}", thought);
            
}
