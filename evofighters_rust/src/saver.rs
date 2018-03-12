use std::fs::File;
use std::io::Error;

use serde_json;

use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use creatures::{Creatures, DeserializableCreatures};
use stats::GlobalStatistics;
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
pub struct Saver {
    filename: String,
    settings: Settings,
}

impl Saver {
    pub const COMPRESSION_LEVEL: u32 = 9;

    pub fn new(filename: &str) -> Saver {
        Saver {
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
        let contents = Checkpoint {
            creatures,
            stats: stats.to_owned(),
            settings: self.settings.to_owned(),
        };
        // Create a writer
        let compressor = XzEncoder::new(
            File::create(&self.filename)?,
            Saver::COMPRESSION_LEVEL,
        );
        serde_json::to_writer(compressor, &contents)?;
        Ok(())
    }

    /// Load a savefile from disk
    pub fn load(filename: &str) -> Result<OwnedCheckpoint, Error> {
        // Create reader
        let decompressor = XzDecoder::new(File::open(filename)?);
        let d_checkpoint: DeserializableCheckpoint =
            serde_json::from_reader(decompressor)?;
        Ok(d_checkpoint.into_owned_checkpoint())
    }
}


/// This checkpoint can be written to disk without needing to take
/// ownership or clone the entire creatures array. It's private
/// because nobody but the Saver should create one of these
/// structures.
#[derive(Serialize)]
struct Checkpoint<'a> {
    creatures: &'a Creatures,
    stats: GlobalStatistics,
    settings: Settings,
}

/// This checkpoint owns its creatures array. It's public because when
/// you load a file this is what's returned.
pub struct OwnedCheckpoint {
    pub creatures: Creatures,
    pub stats: GlobalStatistics,
    pub settings: Settings,
}

/// This checkpoint is what's deserialized from disk. Several of the
/// substructures need to be "hydrated" because parts of their
/// in-memory structures are redundant and aren't serialized to save
/// space. Those redundant portions can be recalculated on the fly
/// from the data that is saved to disk, then an `OwnedCheckpoint` can
/// be returned.
#[derive(Deserialize)]
struct DeserializableCheckpoint {
    pub creatures: DeserializableCreatures,
    pub stats: GlobalStatistics,
    pub settings: Settings,
}

impl DeserializableCheckpoint {
    fn into_owned_checkpoint(self) -> OwnedCheckpoint {
        let DeserializableCheckpoint {
            creatures: deserialized_creatures,
            stats,
            settings,
        } = self;
        OwnedCheckpoint {
            creatures: deserialized_creatures.into_creatures(),
            stats,
            settings,
        }
    }
}
