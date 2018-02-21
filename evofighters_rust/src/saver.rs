use std::fs::File;
use std::io::Error;

use serde_json;

use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use rand::{Rand, Rng, SeedableRng, XorShiftRng};
use rand::distributions;
use rand::distributions::range::SampleRange;

use creatures::{Creature, CreatureID, Creatures};
use settings;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Settings {
    max_thinking_steps: usize,
    max_tree_depth: usize,
    max_inv_size: usize,
    default_energy: usize,
    mating_cost: usize,
    mutation_rate: f64,
    max_gene_value: i8,
    winner_life_bonus: usize,
    max_population_size: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            max_thinking_steps: settings::MAX_THINKING_STEPS,
            max_tree_depth: settings::MAX_TREE_DEPTH,
            max_inv_size: settings::MAX_INV_SIZE,
            default_energy: settings::DEFAULT_ENERGY,
            mating_cost: settings::MATING_COST,
            mutation_rate: settings::MUTATION_RATE,
            max_gene_value: settings::MAX_GENE_VALUE,
            winner_life_bonus: settings::WINNER_LIFE_BONUS,
            max_population_size: settings::MAX_POPULATION_SIZE,
        }
    }
}
#[derive(Debug)]
pub struct SaveFile {
    filename: String,
    settings: Settings,
}

#[derive(Debug, Serialize)]
struct SerializableSaveFile<'a> {
    creatures: &'a Creatures,
    stats: GlobalStatistics,
    settings: Settings,
}
#[derive(Debug, Deserialize)]
pub struct DeserializableSaveFile {
    pub creatures: Creatures,
    pub stats: GlobalStatistics,
    pub settings: Settings,
}

impl SaveFile {
    pub const COMPRESSION_LEVEL: u32 = 9;

    pub fn new(filename: &str) -> SaveFile {
        SaveFile {
            filename: filename.to_owned(),
            settings: Settings::default(),
        }
    }

    /// Save the current file to disk
    pub fn save(
        &mut self,
        creatures: &Creatures,
        stats: &GlobalStatistics,
    ) -> Result<(), Error> {
        let contents = SerializableSaveFile {
            creatures,
            stats: stats.to_owned(),
            settings: self.settings.to_owned(),
        };
        // Create a writer
        let compressor = XzEncoder::new(
            File::create(&self.filename)?,
            SaveFile::COMPRESSION_LEVEL,
        );
        serde_json::to_writer(compressor, &contents)?;
        Ok(())
    }

    /// Load a savefile from disk
    pub fn load(filename: &str) -> Result<DeserializableSaveFile, Error> {
        // Create reader
        let decompressor = XzDecoder::new(File::open(filename)?);
        Ok(serde_json::from_reader(decompressor)?)
    }
}

#[derive(Debug, Clone)]
pub struct RngState {
    rng: XorShiftRng,
}

impl Default for RngState {
    fn default() -> RngState {
        RngState::new(11, 17, 23, 51)
    }
}

impl RngState {
    pub fn new(a: u32, b: u32, c: u32, d: u32) -> RngState {
        RngState {
            rng: SeedableRng::from_seed([a, b, c, d]),
        }
    }

    pub fn from_creatures(a: &Creature, b: &Creature) -> RngState {
        let a_p = CreatureID::parents_to_u32(a.parents);
        let b_p = CreatureID::parents_to_u32(b.parents);
        RngState::new(
            a_p,
            b_p,
            a.hash(),
            b.hash(),
        )
    }

    pub fn spawn(&mut self) -> RngState {
        RngState::new(self.rand(), self.rand(), self.rand(), self.rand())
    }

    pub fn rand<T: Rand>(&mut self) -> T {
        self.rng.gen()
    }

    pub fn rand_range<T: PartialOrd + SampleRange>(
        &mut self,
        low: T,
        high: T,
    ) -> T {
        if low == high {
            low
        } else {
            self.rng.gen_range(low, high)
        }
    }

    pub fn normal_sample(&mut self, mean: f64, std_dev: f64) -> f64 {
        use rand::distributions::IndependentSample;
        let normal = distributions::Normal::new(mean, std_dev);
        normal.ind_sample(&mut self.rng)
    }

    pub fn rand_weighted_bool(&mut self, n: u32) -> bool {
        self.rng.gen_weighted_bool(n)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default)]
pub struct GlobalStatistics {
    pub mutations: usize,
    pub children_born: usize,
    pub feeders_eaten: usize,
    pub kills: usize,
    pub rounds: usize,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Stat {
    Mutations(usize),
    ChildrenBorn(usize),
    FeedersEaten(usize),
    Kills(usize),
    Rounds(usize),
}

impl GlobalStatistics {
    pub fn new() -> GlobalStatistics {
        GlobalStatistics::default()
    }

    pub fn absorb(&mut self, other: GlobalStatistics) {
        self.mutations += other.mutations;
        self.children_born += other.children_born;
        self.feeders_eaten += other.feeders_eaten;
        self.kills += other.kills;
        self.rounds += other.rounds;
    }

    pub fn increment(&mut self, stat: Stat) {
        match stat {
            Stat::Mutations(x) => self.mutations += x,
            Stat::ChildrenBorn(x) => self.children_born += x,
            Stat::FeedersEaten(x) => self.feeders_eaten += x,
            Stat::Kills(x) => self.kills += x,
            Stat::Rounds(x) => self.rounds += x,
        }
    }
}
