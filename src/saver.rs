use std::fs::File;
use std::io::Error;

use serde_json;

use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use creatures::{Creatures, DeserializableCreatures};
use stats::GlobalStatistics;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Builder)]
pub struct Settings {
    #[builder(default = "0.10")]
    pub mutation_rate: f64,

    #[builder(default = "120_000")]
    pub max_population_size: usize,

    #[builder(default = "30.0")]
    pub metric_fps: f64,
}

impl Default for Settings {
    fn default() -> Self {
        SettingsBuilder::default().build().unwrap()
    }
}

impl SettingsBuilder {
    pub fn build_with_defaults(&mut self, other: Settings) -> Settings {
        Settings {
            mutation_rate: self.mutation_rate.unwrap_or(other.mutation_rate),
            max_population_size: self.max_population_size
                .unwrap_or(other.max_population_size),
            metric_fps: self.metric_fps.unwrap_or(other.metric_fps),
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

impl OwnedCheckpoint {
    pub fn new(filename: &str, sb: &mut SettingsBuilder) -> OwnedCheckpoint {
        println!("Attempting to load checkpoint from {}...", filename);
        match Saver::load(filename) {
            Ok(mut checkpoint) => {
                println!(
                    "Success. {} creatures loaded.",
                    checkpoint.creatures.len()
                );
                let settings = sb.build_with_defaults(checkpoint.settings);
                checkpoint.creatures.update_with_settings(&settings);
                checkpoint.settings = settings;
                checkpoint
            }
            Err(_) => {
                let settings = sb.build().unwrap();
                let creatures = Creatures::new(settings.max_population_size);
                println!("Created {} creatures.", settings.max_population_size);
                OwnedCheckpoint {
                    creatures,
                    settings: settings,
                    stats: GlobalStatistics::default(),
                }
            }
        }
    }
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
