use std::thread;
use std::sync::mpsc::channel;
use num_cpus;

use arena::Arena;
use saver::{Saver, Settings};
use creatures::Creatures;
use saver::OwnedCheckpoint;
use stats::GlobalStatistics;

/// Simulation is the coordinating object that manages all of the
/// different threads used to run the sim. It decides how many workers
/// to create, and handles exit signals etc.
///
/// There are X `Worker` threads, where X is the number of physical cores:
///   * `inbox` - a `Receiver` to receive new work to do
///   * `outbox` - a `Sender` channel to send completed work through
///   * `metrics` - a `Sender` channel to send metrics through
///   * `should_exit` - a `Receiver` to get graceful shutdown messages
///
/// There is one `Saver` thread
///   * `checkpoints` - a `Receiver` of checkpoints to save to disk
///   * `metrics` - a `Receiver` of metrics to save to disk
///   * `should_exit` - a `Receiver` to get graceful shutdown messages
///
/// On the main thread:
///   * `worker_out` - a `Vec<Sender>` with channels to send work to `Worker`s
///   * `worker_in` - a `Vec<Receiver>` with channels to receive completed work
///   * `metrics` - a `Sender` channel to send metrics through
///   * `should_exit` - a `Sender` to send graceful shutdown messages
pub struct Simulation {
    filename: String,
    arena: Arena,
    settings: Settings,
}

impl Simulation {
    pub fn new(filename: &str, settings: Settings) -> Simulation {
        let arena = match Saver::load(filename) {
            Ok(checkpoint) => {
                println!("Loading from file {}", filename);
                Arena::from_checkpoint(checkpoint, filename)
            }
            Err(_) => {
                println!("Creating initial population");
                let population: Creatures =
                    Creatures::new(settings.max_population_size);
                println!("Created {} creatures", settings.max_population_size);
                Arena::new(population, filename, settings)
            }
        };

        Simulation {
            filename: filename.to_owned(),
            settings,
            arena,
        }
    }

    pub fn load_or_create(&mut self) -> OwnedCheckpoint {
        println!("Attempting to load checkpoint from {}...", self.filename);
        match Saver::load(&self.filename) {
            Ok(checkpoint) => {
                println!(
                    "Success. {} creatures loaded.",
                    checkpoint.creatures.len()
                );
                checkpoint
            }
            Err(_) => {
                let creatures =
                    Creatures::new(self.settings.max_population_size);
                println!(
                    "Created {} creatures.",
                    self.settings.max_population_size
                );
                OwnedCheckpoint {
                    creatures,
                    settings: self.settings,
                    stats: GlobalStatistics::default(),
                }
            }
        }
    }

    pub fn simulate(&mut self) {
        self.arena.simulate()
    }

    pub fn full_simulate(&mut self) {
        let physical_cores = num_cpus::get_physical();
    }
}
