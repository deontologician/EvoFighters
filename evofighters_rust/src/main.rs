#![feature(rand)]
#![feature(core)]
#![feature(box_syntax)]

use std::rc;

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;

fn main() {
    let mut app = util::AppState::new(1);
    let mut v: Vec<i8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let mut v2: Vec<i8> = vec![0,0,0,0,0,0,0,0];

    for x in v.iter_mut() {
        *x = app.rand_range(-1, 9);
    }
    for y in v2.iter_mut() {
        *y = app.rand_range(-1, 9)
    }

    let mut creature_1 = creatures::Creature::new(1, rc::Rc::new(v), 0, (0,0));
    let mut creature_2 = creatures::Creature::new(2, rc::Rc::new(v2), 0, (0,0));

    creature_1.add_item(dna::Item::GoodFood);
    creature_2.add_item(dna::Item::GoodFood);

    arena::encounter(&mut creature_1, &mut creature_2, &mut app);
}
