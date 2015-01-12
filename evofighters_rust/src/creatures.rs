use dna;
use eval;
use std::rc;

#[derive(Show)]
struct<'a> Creature<'a> {
    dna: &'a[u8],
    _inv: Vec<dna::Item>,
    energy: isize,
    target: rc::Rc<Creature>,
    generation: usize,
    num_children: usize,
    signal: dna::Signal,
    survived: usize,
    kills: usize,
    instr_used: usize,
    instr_skipped: usize,
    last_action: eval::PerformableAction,
    name: String,
    is_feeder: bool,
    eaten: usize,
    parents: (rc::Rc<Creature>, rc::Rc<Creature>),
}
