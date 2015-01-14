use dna;
use eval;
use parsing;
use std::rc;
use std::fmt;
use settings;

#[derive(Show)]
pub struct Creature <'a> {
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
    // These are used for the iterator
    parser: parsing::Parser<'a>,
    thought: parsing::Thought,
}

impl <'a> fmt::String for Creature<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<]Creature {}[>", self.id)
    }
}

impl <'a> Creature<'a> {

    pub fn new(&mut self,
           id: usize,
           dna: &'a [u8],
           parents: (usize, usize)) -> Creature<'a> {
        Creature {
            dna: dna,
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            generation: 0,
            num_children: 0,
            signal: None,
            survived: 0,
            kills: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::Wait,
            id: id,
            is_feeder: false,
            eaten: 0,
            parents: parents,
            parser: parsing::Parser::new(dna),
            thought: parsing::Thought::Indecision {
                reason: parsing::Failure::NoThoughtsYet,
                icount: 0,
                skipped: 0,
            }
        }
    }

    pub fn add_item(&mut self, item: dna::Item) {
        if self.inv.len() + 1 <= settings::MAX_INV_SIZE {
            self.inv.push(item)
        }
    }

    pub fn pop_item(&mut self) -> Option<dna::Item> {
        self.inv.pop()
    }

    pub fn has_items(&self) -> bool {
        !self.inv.is_empty()
    }

    pub fn top_item(&self) -> Option<dna::Item> {
        if !self.inv.is_empty() {
            Some(self.inv[self.inv.len() - 1])
        }else{
            None
        }
    }

    pub fn dead(&self) -> bool {
        self.energy <= 0 || self.dna.is_empty()
    }

    pub fn alive(&self) -> bool {
        !self.dead()
    }
}

impl <'a> Iterator for Creature<'a> {
    type Item = (eval::PerformableAction, usize);
    fn next(&mut self) -> Option<(eval::PerformableAction, usize)> {
        self.thought = self.parser.next().unwrap();
        match self.thought {
            x @ parsing::Thought::Decision{..} => {
                panic!("")
            },
            y @ parsing::Thought::Indecision{..} => {
                panic!("")
            }
        }
    }
}
