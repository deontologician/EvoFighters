use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Receiver, Sender};
use num_cpus;

use arena::{Arena,Encounter,EncounterResult};
use saver::{Settings, SettingsBuilder};
use creatures::{Creature, IDGiver};
use saver::OwnedCheckpoint;
use stats::GlobalStatistics;
use rng::RngState;

type EncounterJob = Vec<(Creature, Creature)>;

/// Simulation is the coordinating object that manages all of the
/// different threads used to run the sim. It decides how many workers
/// to create, and handles exit signals etc.
///
/// There are X `Worker` threads, where X is the number of physical cores:
///   * `inbox` - a `Receiver` to receive new work to do
///   * `outbox` - a `Sender` channel to send completed work through
///   * `metrics` - a `Sender` channel to send metrics through
///
/// There is one `Saver` thread
///   * `checkpoints` - a `Receiver` of checkpoints to save to disk
///   * `metrics` - a `Receiver` of metrics to save to disk
///
/// On the main thread:
///   * `worker_out` - a `Vec<Sender>` with channels to send work to `Worker`s
///   * `worker_in` - a `Vec<Receiver>` with channels to receive completed work
///   * `metrics` - a `Sender` channel to send metrics through
pub struct SingleThreadedSimulation {
    filename: String,
    arena: Arena,
    settings: Settings,
}

impl SingleThreadedSimulation {
    pub fn new(
        filename: &str,
        mut settings_builder: SettingsBuilder,
    ) -> SingleThreadedSimulation {
        let checkpoint = OwnedCheckpoint::new(filename, &mut settings_builder);
        let arena = Arena::from_checkpoint(checkpoint, filename);

        SingleThreadedSimulation {
            filename: filename.to_owned(),
            settings: settings_builder.build().unwrap(),
            arena,
        }
    }

    pub fn simulate(&mut self) {
        self.arena.simulate()
    }
}

pub struct MultiThreadedSimulation {
    filename: String,
    arena: Arena,
    settings: Settings,
}

impl MultiThreadedSimulation {
    pub fn simulate(&mut self) {
        let num_cores = num_cpus::get_physical();
        let (tx_metrics, rx_metrics) = channel();
        let (tx_checkpoint, rx_checkpoint) = channel::<OwnedCheckpoint>();
        let mut id_givers = IDGiver::per_thread(num_cores);
        let mut unspawned_workers = Vec::new();
        let mut rng = RngState::default();

        for worker_id in 0..num_cores {
            let (tx_inbox, rx_inbox) = channel();
            let (tx_outbox, rx_outbox) = channel();
            unspawned_workers.push(UnspawnedWorkerRecord {
                id: worker_id,
                inbox: tx_inbox,
                outbox: rx_outbox,
                worker: Worker {
                    id: worker_id,
                    id_giver: id_givers.pop().unwrap(),
                    rng: rng.spawn(),
                    settings: self.settings,
                    inbox: rx_inbox,
                    outbox: tx_outbox,
                    metrics: tx_metrics.clone(),
                },
            });
        }
    }
}
pub struct Worker {
    pub id: usize,
    pub rng: RngState,
    pub id_giver: IDGiver,
    pub settings: Settings,
    pub inbox: Receiver<EncounterJob>,
    pub outbox: Sender<Vec<Creature>>,
    pub metrics: Sender<GlobalStatistics>,
}

impl Worker {
    pub fn worker_loop(mut self) {
        println!("Worker {} starting up", self.id);
        let id = self.id;
        for encounter_job in self.inbox {
            for (p1, p2) in encounter_job {
                let encounter = Encounter::new(
                    p1,
                    p2,
                    self.settings.mutation_rate,
                    &mut self.rng,
                    &mut self.id_giver,
                );
                let EncounterResult { survivors, stats } = encounter.run();
                self.outbox.send(survivors).unwrap_or_else(|_err| {
                    println!("Worker {} noticed its outbox is dead", id);
                });
                self.metrics.send(stats).unwrap_or_else(|_err| {
                    println!("Worker {} noticed metrics is dead", id);
                });
            }
        }
    }
}

pub struct UnspawnedWorkerRecord {
    pub id: usize,
    pub inbox: Sender<EncounterJob>,
    pub outbox: Receiver<Vec<Creature>>,
    pub worker: Worker,
}

pub struct WorkerRecord {
    id: usize,
    inbox: Sender<EncounterJob>,
    outbox: Receiver<Vec<Creature>>,
    join_handle: JoinHandle<()>,
}

impl UnspawnedWorkerRecord {
    pub fn spawn(self) -> WorkerRecord {
        let UnspawnedWorkerRecord {
            id,
            inbox,
            outbox,
            worker,
        } = self;
        let join_handle = thread::spawn(move || {
            worker.worker_loop()
        });
        WorkerRecord {
            id,
            inbox,
            outbox,
            join_handle,
        }
    }
}
