use std::fmt;
use std::num::{Int, Float};
use std::cmp::{max,min};
use std::rc;
use rustc_serialize::json;

use dna;
use eval;
use parsing;
use settings;
use arena;
use util;

static FEEDER_ID: usize = 0;

#[derive(Show,PartialEq,Eq,Copy)]
pub enum Liveness {
    Alive, Dead
}

#[derive(Show,Clone, RustcDecodable, RustcEncodable)]
pub struct Creature {
    dna: dna::DNA,
    inv: Vec<dna::Item>,
    energy: usize,
    pub generation: usize,
    pub num_children: usize,
    pub signal: Option<dna::Signal>,
    pub survived: usize,
    pub kills: usize,
    pub instr_used: usize,
    pub instr_skipped: usize,
    pub last_action: eval::PerformableAction,
    pub id: usize,
    pub eaten: usize,
    pub parents: (usize, usize),
}


impl fmt::Display for Creature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_feeder() {
            write!(f, "[|Feeder|]")
        } else {
            write!(f, "<]Creature {}[>", self.id)
        }
    }
}

impl Creature {

    pub fn new(id: usize,
               dna: dna::DNA,
               generation: usize,
               parents: (usize, usize)) -> Creature {
        Creature {
            dna: dna.clone(),
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            generation: generation,
            num_children: 0,
            signal: None,
            survived: 0,
            kills: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::Wait,
            id: id,
            eaten: 0,
            parents: parents,
        }
    }

    pub fn iter(&self) -> parsing::Parser {
        parsing::Parser::new(
            self.dna.clone(), self.instr_used + self.instr_skipped)
    }

    pub fn feeder() -> Creature {
        let dntel: dna::DNA = dna::empty_dna();
        Creature {
            id: FEEDER_ID,
            dna: dntel.clone(),
            inv: Vec::new(),
            energy: 1,
            generation: 0,
            num_children: 0,
            signal: Some(dna::Signal::Green),
            kills: 0,
            survived: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::Wait,
            eaten: 0,
            parents: (0, 0),
        }
    }

    pub fn is_feeder(&self) -> bool {
        self.id == FEEDER_ID
    }

    fn has_items(&self) -> bool {
        return !self.inv.is_empty()
    }

    pub fn add_item(&mut self, item: dna::Item) {
        if self.inv.len() < settings::MAX_INV_SIZE {
            self.inv.push(item)
        } else {
            print2!("{} tries to add {:?} but has no more space",
                    self, item);
        }
    }

    pub fn survived_encounter(&mut self) {
        self.survived += 1;
        self.last_action = eval::PerformableAction::Wait;
    }

    fn set_signal(&mut self, signal:dna::Signal) {
        self.signal = Some(signal)
    }

    fn pop_item(&mut self) -> Option<dna::Item> {
        self.inv.pop()
    }

    pub fn top_item(&self) -> Option<dna::Item> {
        if !self.inv.is_empty() {
            Some(self.inv[self.inv.len() - 1])
        } else {
            None
        }
    }

    fn eat(&mut self, item: dna::Item) {
        let energy_gain = 3 * item as usize;
        print2!("{} gains {} life from {:?}", self, energy_gain, item);
        self.gain_energy(energy_gain)
    }

    pub fn liveness(&self) -> Liveness {
        use self::Liveness::{Alive, Dead};
        if self.is_feeder() {
            if self.energy > 0 && self.has_items() {
                Alive
            } else {
                Dead
            }
        } else {
            if self.energy > 0 && !self.dna.is_empty() {
                Alive
            } else {
                Dead
            }
        }
    }

    pub fn dead(&self) -> bool {
        self.liveness() == Liveness::Dead
    }

    pub fn steal_from(&mut self, other: &mut Creature) {
        if let Some(item) = other.pop_item() {
            self.add_item(item)
        }
    }

    pub fn energy(&self) -> usize {
        self.energy
    }

    pub fn lose_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount)
    }

    pub fn gain_energy(&mut self, amount: usize) {
        self.energy += amount;
        self.energy = min(settings::DEFAULT_ENERGY, self.energy);
    }

    fn kill(&mut self) {
        self.energy = 0;
    }

    pub fn update_from_thought(&mut self, thought: &parsing::Thought) {
        self.instr_used += thought.skipped();
        self.instr_skipped += thought.skipped();
        if !thought.decided() {
            self.kill()
        }
    }

    pub fn carryout(&mut self,
                    other: &mut Creature,
                    action: eval::PerformableAction,
                    app: &mut util::AppState) -> arena::FightStatus {
        if self.is_feeder() {
            print2!("Feeder does nothing");
            return arena::FightStatus::Continue;
        }
        if self.dead() {
            return arena::FightStatus::End
        }
        match action {
            eval::PerformableAction::Signal(sig) => {
                self.set_signal(sig)
            },
            eval::PerformableAction::Eat => {
                match self.pop_item() {
                    Some(item) => {
                        print1!("{} eats {:?}",
                                self, self.top_item());
                        self.eat(item);
                    },
                    None =>
                        print2!("{} tries to eat an item, but \
                                doesn't have one", self)
                }
            },
            eval::PerformableAction::Take => {
                match other.pop_item() {
                    Some(item) => {
                        print1!("{} takes {:?} from {}",
                                self, item, other.id);
                        self.add_item(item);
                    },
                    None => {
                        print2!("{} tries to take an item from {}, \
                                but there's nothing to take.",
                                self, other.id);
                    }
                }
            },
            eval::PerformableAction::Wait => print2!(
                "{} waits", self),
            // This is only defending with no corresponding attack
            eval::PerformableAction::Defend(dmg) =>
                print2!("{} defends with {:?} fruitlessly",
                        self, dmg),
            eval::PerformableAction::Flee => {
                let my_roll: f64 = app.rand_range(0.0, self.energy as f64);
                let other_roll: f64 = app.rand_range(0.0, other.energy as f64);
                let dmg: usize = app.rand_range(0, 4);
                if other_roll < my_roll {
                    print1!("{} flees the encounter and takes \
                            {} damage", self, dmg);
                    self.lose_energy(dmg);
                    return arena::FightStatus::End
                } else {
                    print2!("{} tries to flee, but {} prevents it",
                            self, other);
                }

            },
            invalid_action => panic!("Shouldn't have gotten {:?} here",
                                     invalid_action)
        }
        arena::FightStatus::Continue
    }
}


fn gene_primer(dna: dna::DNA) -> Vec<Vec<i8>> {
    let mut result = Vec::new();
    let mut chunk = Vec::new();
    for &base in dna.iter() {
        chunk.push(base);
        if base == -1 {
            result.push(chunk);
            chunk = Vec::new();
        }
    }
    if !chunk.is_empty() {
        result.push(chunk);
    }
    result
}

pub fn try_to_mate(
    mating_chance: usize,
    first_mate: &mut Creature,
    first_share: usize,
    second_mate: &mut Creature,
    second_share: usize,
    app: &mut util::AppState) -> Option<Creature> {
    if app.rand_range(0, 100) > mating_chance
        || first_mate.dead()
        || second_mate.dead() {
            return None
        }
    print1!("{} tried to mate with {}!", first_mate, second_mate);
    if first_mate.is_feeder() || second_mate.is_feeder() {
        print1!("{} tried to mate with {}", first_mate, second_mate);
        // Mating kills the feeder
        if first_mate.is_feeder() {
            first_mate.energy = 0;
        }
        if second_mate.is_feeder() {
            second_mate.energy = 0;
        }
        return None
    }
    print2!("Attempting to mate");
    fn pay_cost(p: &mut Creature, share: usize) -> bool {
        let mut cost = (settings::MATING_COST as f64 *
                    (share as f64 / 100.0)).round() as isize;
        while cost > 0 {
            match p.pop_item() {
                Some(item) => {
                    cost -= (item as isize) * 2;
                },
                None => {
                    print1!("{} ran out of items and failed to mate", p);
                    return false
                }
            }
        }
        true
    }
    if pay_cost(first_mate, first_share) &&
        pay_cost(second_mate, second_share) {
        Some(mate(first_mate, second_mate, app))
    } else {
        None
    }
}

fn mate(p1: &mut Creature,
            p2: &mut Creature,
            app: &mut util::AppState) -> Creature {
    let dna1_primer = gene_primer(p1.dna.clone());
    let dna2_primer = gene_primer(p2.dna.clone());
    let child_gene_len = max(dna1_primer.len(), dna2_primer.len());
    let mut dna1 = dna1_primer.into_iter();
    let mut dna2 = dna2_primer.into_iter();
    let mut child_genes : Vec<Vec<i8>> = Vec::with_capacity(child_gene_len + 1);
    loop {
        let gene1 = dna1.next().unwrap_or(Vec::new());
        let gene2 = dna2.next().unwrap_or(Vec::new());
        if gene1.is_empty() && gene2.is_empty() {
            break;
        }
        child_genes.push(if app.rand() {
            gene1
        } else {
            gene2
        });
    }
    if app.rand_range(0.0, 1.0) < settings::MUTATION_RATE {
        mutate(&mut child_genes, app)
    }
    let mut dna_vec = Vec::with_capacity(child_genes.len() * 12);
    for gene in child_genes.into_iter() {
        dna_vec.extend(gene.into_iter())
    }

    let child = Creature::new(
        app.next_creature_id(), // id
        rc::Rc::new(dna_vec), // dna
        max(p1.generation, p2.generation) + 1, // generation
        (p1.id, p2.id), // parents
        );
    p1.num_children += 1;
    p2.num_children += 1;
    child
}

fn mutate(genes: &mut Vec<Vec<i8>>, app: &mut util::AppState) {
    if app.rand_weighted_bool(
        (10000.0/settings::MUTATION_RATE) as usize) {
        genome_level_mutation(genes, app);
    } else {
        let index = app.rand_range(0, genes.len());
        let fixed_gene = &mut genes[index];
        print2!("Mutating gene {}", index);
        gene_level_mutation(fixed_gene, app);
    }
}

fn genome_level_mutation(
    genome: &mut Vec<Vec<i8>>,
    app: &mut util::AppState) {
    match app.rand_range(1, 4) {
        1 => { // swap two genes
            let i1 = app.rand_range(0, genome.len());
            let i2 = app.rand_range(0, genome.len());
            print2!("swapped genes {} and {}", i1, i2);
            genome.as_mut_slice().swap(i1, i2);
        },
        2 => { // double a gene
            let i = app.rand_range(0, genome.len());
            let gene = genome[i].clone();
            print2!("doubled gene {}", i);
            genome.insert(i, gene);
        },
        3 => { // deletes a gene
            let i = app.rand_range(0, genome.len());
            print2!("Deleted gene {}", i);
            // Avoid shifting items if we can, we're going to flatten
            // this list anyway later
            genome.push(Vec::new());
            genome.swap_remove(i);
        },
        _ => panic!("Generated in range 1 - 3! Should not reach.")
    }
}

fn gene_level_mutation(gene: &mut Vec<i8>, app: &mut util::AppState) {
    if gene.is_empty() {
        print3!("Mutated an empty gene!");
        return
    }
    match app.rand_range(1, 6) {
        1 => { // reverse the order of bases in a gene
            gene.as_mut_slice().reverse();
            print2!("reversed gene");
        },
        2 => { // deleting a gene
            gene.clear();
            print2!("deleted gene");
        },
        3 => { // insert an extra base in a gene
            let val = app.rand_range(-1, settings::MAX_GENE_VALUE);
            let index = app.rand_range(0, gene.len());
            print2!("inserted {} at {}", val, index);
            gene.insert(index, val);
        },
        4 => { // increment a base in a gene, modulo the
            // max gene value
            let inc = app.rand_range(1, 3);
            let index = app.rand_range(0, gene.len());
            let new_base = (gene[index] + 1 + inc) %
                (settings::MAX_GENE_VALUE + 2) - 1;
            print2!("added {} to base at {} with val {} to get {}",
                    inc, index, gene[index], new_base);
            gene[index] = new_base;
        },
        5 => { // swap two bases in the gene
            let i1 = app.rand_range(0, gene.len());
            let i2 = app.rand_range(0, gene.len());
            gene.as_mut_slice().swap(i1, i2);
            print2!("swapped bases {} and {}", i1, i2);
        },
        _ => panic!("Impossible. number between 1 and 6 exclusive")
    }
}
