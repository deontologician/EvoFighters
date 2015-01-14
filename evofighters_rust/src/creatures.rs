use dna;
use eval;
use std::rc;
use std::fmt;
use std::sync::atomic::{AtomicUsize,Ordering,ATOMIC_USIZE_INIT};
use settings;

#[derive(Show)]
struct Creature <'a> {
    dna: &'a [u8],
    inv: Vec<dna::Item>,
    pub energy: isize,
    pub generation: usize,
    // Omitted target, will deal with elsewhere. No reason for a
    // creature to own another creature really, they just need to be
    // present at the same time.
    pub num_children: usize,
    pub signal: Option<dna::Signal>,
    pub survived: usize,
    pub kills: usize,
    pub instr_used: usize,
    pub instr_skipped: usize,
    pub last_action: eval::PerformableAction,
    pub id: usize,
    pub is_feeder: bool,
    pub eaten: usize,
    pub parents: (usize, usize),
}

impl <'a> fmt::String for Creature<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<]Creature {}[>", self.id)
    }
}

static mut CreatureCount: AtomicUsize = ATOMIC_USIZE_INIT;

impl <'a> Creature<'a> {

    fn new(&mut self, dna: &'a [u8], parents: (usize, usize)) -> Creature<'a> {
        Creature {
            dna: dna,
            inv: vec![],
            energy: settings::DEFAULT_ENERGY,
            generation: 0,
            num_children: 0,
            signal: None,
            survived: 0,
            kills: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::Wait,
            id: CreatureCount.fetch_add(1us, Ordering::Relaxed),
            is_feeder: false,
            eaten: 0,
            parents: parents,
        }
    }

    fn add_item(&mut self, item: dna::Item) {
        if self.inv.len() + 1 <= settings::MAX_INV_SIZE {
            self.inv.push(item)
        }
    }

}
