use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Receiver, Sender};
use num_cpus;

use arena::Arena;
use saver::{Settings, SettingsBuilder};
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
        let mut unspawned_workers = Vec::new();

        for worker_id in 0..num_cores {
            let (tx_inbox, rx_inbox) = channel();
            let (tx_outbox, rx_outbox) = channel();
            let (tx_exit, rx_exit) = channel();
            unspawned_workers.push(UnspawnedWorkerRecord {
                id: worker_id,
                inbox: tx_inbox,
                outbox: rx_outbox,
                should_exit: tx_exit,
                worker_state: WorkerState {
                    id: worker_id,
                    inbox: rx_inbox,
                    outbox: tx_outbox,
                    metrics: tx_metrics.clone(),
                    should_exit: rx_exit,
                },
            });
        }
    }
}
pub struct WorkerState {
    pub id: usize,
    pub inbox: Receiver<Creatures>,
    pub outbox: Sender<Creatures>,
    pub metrics: Sender<GlobalStatistics>,
    pub should_exit: Receiver<bool>,
}

pub struct UnspawnedWorkerRecord {
    pub id: usize,
    pub inbox: Sender<Creatures>,
    pub outbox: Receiver<Creatures>,
    pub should_exit: Sender<bool>,
    pub worker_state: WorkerState,
}

pub struct WorkerRecord {
    id: usize,
    inbox: Sender<Creatures>,
    outbox: Receiver<Creatures>,
    should_exit: Sender<bool>,
    join_handle: JoinHandle<()>,
}

impl UnspawnedWorkerRecord {
    pub fn spawn<F>(self) -> WorkerRecord
    where
        F: FnOnce() -> (),
        F: Send + 'static,
    {
        let UnspawnedWorkerRecord {
            id,
            inbox,
            outbox,
            should_exit,
            worker_state,
        } = self;
        let join_handle = thread::spawn(move || {worker_state;});
        WorkerRecord {
            id,
            inbox,
            outbox,
            should_exit,
            join_handle,
        }
    }
}
