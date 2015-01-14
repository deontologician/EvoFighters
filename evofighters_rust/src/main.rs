#![allow(unstable)]

pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;

fn main() {
    let mut x : Vec<usize> = Vec::with_capacity(3);
    x.push(3us);
    x.push(4us);
    println!("It works! {}", x[x.len() - 1]);
}
