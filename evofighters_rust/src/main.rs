#![allow(unstable)]

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;

fn main() {
    let x: Vec<u8> = vec![1,2,1,1,1,1,1,1,1,-1];
    let mut parser = parsing::Parser::new(x.as_slice());
    let thought = parser.next().expect("No thought!");
    match thought {
        parsing::Thought::Decision {tree, icount, skipped} =>
            println!("icount: {}, skipped: {}, tree:\n{:?}",
                     icount, skipped, tree),
        _ => panic!("Bo.")
    }
            
}
