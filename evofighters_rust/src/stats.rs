use std::time::{Duration, Instant};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default)]
pub struct GlobalStatistics {
    pub children_born: usize,
    pub feeders_eaten: usize,
    pub kills: usize,
    pub rounds: usize,
    pub encounters: usize,
}

impl GlobalStatistics {
    pub fn new() -> GlobalStatistics {
        GlobalStatistics::default()
    }

    pub fn absorb(&mut self, EncounterStats {
        children_born,
        feeders_eaten,
        kills,
        rounds,
    }: EncounterStats) {
        self.children_born += children_born;
        self.feeders_eaten += feeders_eaten;
        self.kills += kills;
        self.rounds += rounds;
        self.encounters += 1;
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreatureStats {
    pub kills: usize,
    pub num_children: usize,
    pub survived: usize,
    pub eaten: usize,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct EncounterStats {
    pub children_born: usize,
    pub feeders_eaten: usize,
    pub kills: usize,
    pub rounds: usize,
}


#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct WorkerStats {
    pub encounters_per_second: f64,
    pub rounds_per_second: f64,
    pub kills_per_second: f64,
    pub children_born_per_second: f64,
    pub feeders_eaten_per_second: f64,
}

impl WorkerStats {
}

/// An helper type that can be used to decide when to run a particular
/// callback at a desired frequency. It caches the number of events to wait before doing anything so that
struct CallbackChecker {
    desired_fps: f64,
    events_until_next_call: u64,
}

/// Given an instant and how many events the thread slept for, will
/// return how many events the thread should sleep for next time
pub struct TimeKeeper {
    events_since_last_syscall: u64,
    events_until_next_syscall: u64,

    last_instant: Instant,

    desired_fps: f64,
    actual_fps: f64,
    listeners: Vec<CallbackChecker>,
}

impl TimeKeeper {
    pub fn new(
        desired_syscall_fps: f64,
        initial_events_per_second_guess: u64,
    ) -> TimeKeeper {
        let events_until_next_syscall =
            initial_events_per_second_guess / (desired_syscall_fps as u64);
        TimeKeeper {
            events_since_last_syscall: 0,
            events_until_next_syscall,
            last_instant: Instant::now(),
            desired_fps,
            actual_fps: desired_fps,
        }
    }

    pub fn fps(&self) -> f64 {
        self.actual_fps
    }

    pub fn increment_events(&mut self) {
        self.events_since_last_syscall += 1;
        if self.events_since_last_syscall >= self.events_until_next_syscall {
            self.do_syscall_and_recalculate();
        }
    }

    fn do_syscall_and_recalculate(&mut self)  {
        let desired_duration = self.desired_fps.recip();
        let actual_duration = self.seconds_since_last_syscall();
        let events_per_second =
            (self.events_until_next_syscall as f64) / actual_duration;

        // Set everything
        self.events_since_last_syscall = 0;
        self.events_until_next_syscall = (desired_duration * events_per_second) as u64;

        self.actual_fps = actual_duration.recip();

        self.last_instant = Instant::now();
    }

    /// Takes a standard duration and returns an f64 representing
    /// seconds
    fn seconds_since_last_syscall(&self) -> f64 {
        let dur = self.last_instant.elapsed();
        let seconds = dur.as_secs() as f64;
        let subseconds = f64::from(dur.subsec_nanos()) / 1_000_000_000.0;
        seconds + subseconds
    }
}
