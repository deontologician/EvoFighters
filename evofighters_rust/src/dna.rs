use std::convert::From;
use std::ops::{Index, IndexMut};
use std::mem;
use std::hash::Hasher;
use std::slice::Iter;

use twox_hash::XxHash32;

use settings;
use stats::GlobalStatistics;
use rng::RngState;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Gene([i8; 5]);

impl Gene {
    pub const STOP_CODON: i8 = -1;
    pub const LENGTH: usize = 5;

    /// Produces a gene full of all stop codons. Useful for allocation
    /// then overwriting
    pub fn new() -> Gene {
        Gene([
            Gene::STOP_CODON,
            Gene::STOP_CODON,
            Gene::STOP_CODON,
            Gene::STOP_CODON,
            Gene::STOP_CODON,
        ])
    }

    /// Creates a gene for mating then fleeing. This is the initial gene.
    pub fn mate_then_flee() -> Gene {
        Gene([
            lex::Condition::Always as i8,
            lex::Action::Mate as i8,
            Gene::STOP_CODON,
            lex::Condition::Always as i8,
            lex::Action::Flee as i8,
        ])
    }

    pub fn always_wait() -> Gene {
        let mut gene = Gene::new();
        gene.0[0] = lex::Condition::Always as i8;
        gene.0[1] = lex::Action::Wait as i8;
        gene
    }

    /// Whether the current Gene codes anything useful
    pub fn valid(&self) -> bool {
        self.0.iter().any(|&codon| codon != Gene::STOP_CODON)
    }

    /// Inversion of valid, to make code read better
    pub fn invalid(&self) -> bool {
        self.0.iter().all(|&codon| codon == Gene::STOP_CODON)
    }

    /// Sets all bases in the gene to the stop codon (making it
    /// invalid)
    fn clear(&mut self) {
        for elem in &mut self.0 {
            *elem = Gene::STOP_CODON
        }
    }

    pub fn iter(&self) -> Iter<i8> {
        self.0.iter()
    }

    pub(super) fn mutate(&mut self, rng: &mut RngState) -> Option<Gene> {
        match rng.rand_range(1, 6) {
            1 => {
                // reverse the order of bases in a gene
                self.0.reverse();
                debug!("reversed gene");
                None
            }
            2 => {
                // deleting a gene
                self.clear();
                debug!("deleted gene");
                None
            }
            3 => {
                // Create a new gene, and set one base in it to a random value
                let index = rng.rand_range(0, Gene::LENGTH);
                let val =
                    rng.rand_range(Gene::STOP_CODON, settings::MAX_GENE_VALUE);
                debug!(
                    "created a new gene with base {} at index {}",
                    val, index
                );
                let mut new_gene = Gene::new();
                new_gene.0[index] = val;
                Some(new_gene)
            }
            4 => {
                // increment a base in a gene, modulo the
                // max gene value
                let inc = rng.rand_range(1, 3);
                let index = rng.rand_range(0, Gene::LENGTH);
                let new_base = (self.0[index] + 1 + inc)
                    % (settings::MAX_GENE_VALUE + 2)
                    - 1;
                debug!(
                    "added {} to base at {} with val {} to get {}",
                    inc, index, self.0[index], new_base
                );
                self.0[index] = new_base;
                None
            }
            5 => {
                // swap two bases in the gene
                let i1 = rng.rand_range(0, Gene::LENGTH);
                let i2 = rng.rand_range(0, Gene::LENGTH);
                self.0.swap(i1, i2);
                debug!("swapped bases {} and {}", i1, i2);
                None
            }
            _ => unreachable!(),
        }
    }
}

impl Default for Gene {
    fn default() -> Self {
        Gene::new()
    }
}

impl Index<usize> for Gene {
    type Output = i8;

    fn index(&self, index: usize) -> &i8 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Gene {
    fn index_mut(&mut self, index: usize) -> &mut i8 {
        &mut self.0[index]
    }
}

/// Core DNA data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNA(Vec<Gene>);

impl DNA {
    /// Produce feeder DNA, which is just stop codons
    pub fn feeder() -> DNA {
        DNA(vec![Gene::always_wait()])
    }

    /// Produce the default seed DNA, which evaluates to "always mate,
    /// always flee"
    pub fn seed() -> DNA {
        DNA(vec![Gene::mate_then_flee()])
    }

    pub fn len(&self) -> usize {
        self.0.len() * Gene::LENGTH
    }

    pub fn base_stream(&self, offset: usize) -> DNAIter {
        DNAIter::new(self.clone(), offset)
    }

    pub fn valid(&self) -> bool {
        !self.0.is_empty() && self.0.iter().all(|&gene| gene.valid())
    }

    pub fn combine(
        mother: &DNA,
        father: &DNA,
        rng: &mut RngState,
    ) -> (DNA, GlobalStatistics) {
        let mut m_iter = mother.0.clone().into_iter();
        let mut f_iter = father.0.clone().into_iter();
        let mut child_genes = Vec::new();
        let mut stats = GlobalStatistics::new();
        // TODO: This code is lousy with unnecessary allocations,
        // clean this up a bit, use more copies / references if possible
        loop {
            let gene1 = m_iter.next().unwrap_or_else(Gene::new);
            let gene2 = f_iter.next().unwrap_or_else(Gene::new);
            if gene1.invalid() && gene2.invalid() {
                break;
            }
            child_genes.push(if rng.rand() { gene1 } else { gene2 });
        }
        if rng.rand_range(0.0, 1.0) < settings::MUTATION_RATE {
            DNA::mutate(&mut child_genes, rng);
            stats.mutations += 1;
        }
        (DNA(child_genes), stats)
    }

    fn mutate(genes: &mut Vec<Gene>, rng: &mut RngState) {
        if rng.rand_weighted_bool((10000.0 / settings::MUTATION_RATE) as u32) {
            DNA::genome_level_mutation(genes, rng)
        } else {
            let index = rng.rand_range(0, genes.len());
            let gene_to_mutate = &mut genes[index];
            debug!("Mutating gene {}", index);
            if let Some(new_gene) = gene_to_mutate.mutate(rng) {
                // Gene mutation produced a new gene, so push it in
                // after the current one
                genes.insert(index, new_gene)
            }
        }
    }

    fn genome_level_mutation(genome: &mut Vec<Gene>, rng: &mut RngState) {
        match rng.rand_range(1, 4) {
            1 => {
                // swap two genes
                let i1 = rng.rand_range(0, genome.len());
                let i2 = rng.rand_range(0, genome.len());
                debug!("swapped genes {} and {}", i1, i2);
                genome.as_mut_slice().swap(i1, i2);
            }
            2 => {
                // double a gene
                let i = rng.rand_range(0, genome.len());
                let gene = genome[i];
                debug!("doubled gene {}", i);
                genome.insert(i, gene);
            }
            3 => {
                // deletes a gene
                let i = rng.rand_range(0, genome.len());
                debug!("Deleted gene {}", i);
                // Avoid shifting items if we can
                genome.remove(i);
            }
            _ => panic!("Generated in range 1 - 3! Should not reach."),
        }
    }

    pub fn hash(&self) -> u32 {
        self.seeded_hash(17)
    }

    pub fn seeded_hash(&self, seed: u32) -> u32 {
        let mut hasher = XxHash32::with_seed(seed);
        for gene in &self.0 {
            for base in gene.iter() {
                hasher.write_i8(*base)
            }
        }
        hasher.finish() as u32
    }
}

impl From<Vec<i8>> for DNA {
    fn from(other: Vec<i8>) -> DNA {
        let capacity_needed = other.len()
            + if other.len() / Gene::LENGTH == 0 {
                0
            } else {
                1
            };
        let mut newvec: Vec<Gene> = Vec::with_capacity(capacity_needed);
        let mut current_gene = Gene::new();
        let mut current_index = 0;
        for item in other {
            if current_index >= Gene::LENGTH {
                newvec.push(mem::replace(&mut current_gene, Gene::new()));
                current_index = 0;
            }
            current_gene[current_index] = item;
            current_index += 1;
        }
        newvec.push(current_gene);
        DNA(newvec)
    }
}

#[derive(Debug, Clone)]
pub struct DNAIter {
    dna: Vec<Gene>,
    offset: usize,
    dna_len: usize,
}

impl DNAIter {
    fn new(dna: DNA, offset: usize) -> DNAIter {
        let len = dna.len();
        DNAIter {
            dna: dna.0,
            offset: offset % len,
            dna_len: len,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl Iterator for DNAIter {
    type Item = i8;
    fn next(&mut self) -> Option<i8> {
        let gene_offset = self.offset / Gene::LENGTH;
        let codon_offset = self.offset % Gene::LENGTH;
        let ret = Some(self.dna[gene_offset].0[codon_offset]);
        self.offset = (self.offset + 1) % self.dna_len;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.dna_len, None)
    }
}

/// The lexical module is for raw enums that are used as tokens from
/// the `DNA`, and are fed to the parser.
pub mod lex {
    use std::fmt;

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
        /// Conditions are parsed from `DNA` and specify a test to do
        /// at fight time
        pub enum Condition {
            /// Always do the specified action
            Always,
            /// Do the specified action if the target value is in a
            /// particular range
            InRange,
            /// Do the specified action if the target value is less
            /// than another value
            LessThan,
            /// Do the specified action if the target value is greater
            /// than another value
            GreaterThan,
            /// Do the specified action if the target value is equal
            /// to another value
            EqualTo,
            /// Do the specified action if the target value is not
            /// equal to another value
            NotEqualTo,
            /// Do the specified action if my last action is the specified value
            MyLastAction,
            /// Do the specified action if the other fighter's last action is
            /// the specified value
            OtherLastAction,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
        /// Values are parsed from `DNA` and specify how to get a value at fight time
        pub enum Value {
            /// A literal value is hardcoded in the `DNA` itself
            Literal,
            /// A random value will be generated each time
            Random,
            /// An attribute from the current fighter will be used as the value
            Me,
            /// An attribute from the opponent will be used as the value
            Other,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
        /// Actions are parsed from `DNA` and specify an action to take
        pub enum Action {
            /// Subconditions check something and fork into two possible actions to take
            Subcondition,
            /// Attack the opponent
            Attack,
            /// Mate with the opponent
            Mate,
            /// Defend against the opponent (who may or may not be attacking)
            Defend,
            /// Attempt to eat an item from your inventory
            Eat,
            /// Signal a color to the opponent
            Signal,
            /// Attempt to take something from the opponent
            Take,
            /// Don't do anything
            Wait,
            /// Attempt to flee the encounter
            Flee,
            // If adding an action, update settings::MAX_GENE_VALUE to match
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
        /// Attributes are parsed from `DNA`. When a `Value` requires looking
        /// at a fighter's attributes, this decides which one is selected
        pub enum Attribute {
            /// the value of the fighter's energy
            Energy,
            /// The value of the signal the fighter is signalling
            Signal,
            /// The value of the generation the fighter belongs to
            Generation,
            /// The number of kills the fighter has
            Kills,
            /// The number of fights the fighter has survived
            Survived,
            /// The number of children the fighter has sired
            NumChildren,
            /// The value of the top item in the fighter's inventory
            TopItem,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    impl fmt::Display for Attribute {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Attribute::Energy => write!(f, "energy"),
                Attribute::Signal => write!(f, "signal"),
                Attribute::Generation => write!(f, "generation"),
                Attribute::Kills => write!(f, "kills"),
                Attribute::Survived => write!(f, "encounters survived"),
                Attribute::NumChildren => write!(f, "number of children"),
                Attribute::TopItem => write!(f, "top inventory item"),
            }
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
        /// Parsed from `DNA`, this represents the value of an item in the inventory
        pub enum Item {
            Food = 1,
            GoodFood,
            BetterFood,
            ExcellentFood,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
        /// Parsed from `DNA`, this represents the color of a signal
        pub enum Signal {
            Red = 1,
            Yellow,
            Blue,
            Purple,
            Orange,
            Green,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
        /// Parsed from `DNA`, this represents a damage type
        pub enum DamageType {
            /// Fire damage
            Fire,
            /// Ice damage
            Ice,
            /// Electricity damage
            Electricity,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }
}

/// The `ast` module is structured trees of conditions and actions
/// that need to be evaluated at fight time in order to determine
/// which action the fighter should take. Unlike the `lex` module,
/// these are not simply tokens.
pub mod ast {
    use std::fmt;
    use dna::lex;

    #[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
    pub enum BinOp {
        LT,
        GT,
        EQ,
        NE,
    }

    impl fmt::Display for BinOp {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                BinOp::LT => write!(f, "less than"),
                BinOp::GT => write!(f, "greater than"),
                BinOp::EQ => write!(f, "equal to"),
                BinOp::NE => write!(f, "not equal to"),
            }
        }
    }

    #[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
    pub enum ActorType {
        Me,
        Other,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub enum Condition {
        Always(Action),
        RangeCompare {
            value: Value,
            bound_a: Value,
            bound_b: Value,
            affirmed: Action,
            denied: Action,
        },
        BinCompare {
            operation: BinOp,
            lhs: Value,
            rhs: Value,
            affirmed: Action,
            denied: Action,
        },
        ActionCompare {
            actor_type: ActorType,
            action: Action,
            affirmed: Action,
            denied: Action,
        },
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum Value {
        Literal(u8),
        Random,
        Me(lex::Attribute),
        Other(lex::Attribute),
    }

    impl fmt::Display for Value {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Value::Literal(lit) => write!(f, "{}", lit),
                Value::Random => write!(f, "a random number"),
                Value::Me(ref attr) => write!(f, "my {}", attr),
                Value::Other(ref attr) => write!(f, "my target's {}", attr),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub enum Action {
        Subcondition(Box<Condition>),
        Attack(lex::DamageType),
        Defend(lex::DamageType),
        Signal(lex::Signal),
        Eat,
        Take,
        Mate,
        Wait,
        Flee,
    }
}
