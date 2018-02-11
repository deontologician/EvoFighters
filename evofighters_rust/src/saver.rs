use serde_json;
use std::fs::File;
use std::io::Error;
use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use rand;
use rand::{Rng,ThreadRng,Rand};
use rand::distributions;
use rand::distributions::range::SampleRange;

use creatures::Creature;
use settings;

#[derive(Debug,Deserialize,Serialize)]
pub struct SaveFile {
    max_thinking_steps: usize,
    max_tree_depth: usize,
    max_inv_size: usize,
    default_energy: usize,
    mating_cost: usize,
    mutation_rate: f64,
    max_gene_value: i8,
    winner_life_bonus: usize,
    max_population_size: usize,
    gene_min_size: usize,
    num_encounters: usize,
    feeder_count: usize,
    creatures: Vec<Creature>,
}

impl SaveFile {
    pub const COMPRESSION_LEVEL: u32 = 9;

    pub fn new(creatures: &[Creature],
               feeder_count: usize,
               num_encounters: usize) -> SaveFile {
        SaveFile {
            max_thinking_steps: settings::MAX_THINKING_STEPS,
            max_tree_depth: settings::MAX_TREE_DEPTH,
            max_inv_size: settings::MAX_INV_SIZE,
            default_energy: settings::DEFAULT_ENERGY,
            mating_cost: settings::MATING_COST,
            mutation_rate: settings::MUTATION_RATE,
            max_gene_value: settings::MAX_GENE_VALUE,
            winner_life_bonus: settings::WINNER_LIFE_BONUS,
            max_population_size: settings::MAX_POPULATION_SIZE,
            gene_min_size: settings::GENE_MIN_SIZE,
            num_encounters: num_encounters,
            feeder_count: feeder_count,
            creatures: creatures.to_owned(),
        }
    }

    /// Save the current file to disk
    pub fn save(&self, filename: &str) -> Result<(), Error> {
        // Create a writer
        let compressor = XzEncoder::new(
            File::create(filename)?,
            SaveFile::COMPRESSION_LEVEL,
        );
        serde_json::to_writer(compressor, self)?;
        Ok(())
    }

    pub fn update(&mut self,
                  creatures: &[Creature],
                  feeder_count: usize,
                  num_encounters: usize) {
        self.creatures = creatures.to_owned();
        self.feeder_count = feeder_count;
        self.num_encounters = num_encounters;
    }

    /// Load a savefile from disk
    pub fn load(filename: &str) -> Result<SaveFile, Error> {
        // Create reader
        let decompressor = XzDecoder::new(File::open(filename)?);
        Ok(serde_json::from_reader(decompressor)?)
    }
}

pub struct AppGlobalState {
    id_box: usize
}

impl AppGlobalState {
    pub fn next_creature_id(&mut self) -> usize {
        self.id_box += 1;
        self.id_box
    }
}

pub struct RngState {
    rng: Option<ThreadRng>
}

impl RngState {
    fn rng(&mut self) -> &mut ThreadRng {
        if self.rng.is_none() {
            self.rng = Some(rand::thread_rng())
        }
        &mut self.rng.unwrap()
    }

    pub fn rand<T: Rand>(&mut self) -> T {
        self.rng().gen()
    }

    pub fn rand_range<T: PartialOrd + SampleRange>(
        &mut self, low: T, high: T) -> T {
        if low == high {
            low
        } else {
            self.rng().gen_range(low, high)
        }
    }

    pub fn normal_sample(&mut self, mean: f64, std_dev: f64) -> f64 {
        use rand::distributions::IndependentSample;
        let normal = distributions::Normal::new(mean, std_dev);
        normal.ind_sample(self.rng())
    }

    pub fn rand_weighted_bool(&mut self, n: u32) -> bool {
        self.rng().gen_weighted_bool(n)
    }
}

pub struct Statistics {
    pub mutations: usize,
    pub children_born: usize,
    pub feeders_eaten: usize,
    pub kills: usize,
    pub rounds: usize,
}

pub enum Stat {
    Mutations(usize),
    ChildrenBorn(usize),
    FeedersEaten(usize),
    Kills(usize),
    Rounds(usize),
}

impl Statistics {
    pub fn new() {
        Statistics {
            mutations: 0,
            children_born: 0,
            feeders_eaten: 0,
            kills: 0,
            rounds: 0,
        }
    }

    pub fn increment(&mut self, stat: Stat) {
        match stat {
            Mutations(x) => self.mutations += x,
            ChildrenBorn(x) => self.children_born += x,
            FeedersEaten(x) => self.feeders_eaten += x,
            Kills(x) => self.kills += x,
            Rounds(x) => self.rounds += x,
        }
    }
}
