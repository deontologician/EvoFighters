use rand;
use rand::{Rng,ThreadRng,Rand};
use rand::distributions;
use rand::distributions::range::SampleRange;

// TODO: add the stars to different debug statements
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => (
        if cfg!(feature = "log_trace") ||
            cfg!(feature = "log_debug") ||
            cfg!(feature = "log_info") {
            println!($($arg)*);
        })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (
        if cfg!(feature = "log_info") || cfg!(feature = "log_debug") {
            println!($($arg)*);
        })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        if cfg!(feature = "log_info") {
            println!($($arg)*);
        })
}

pub struct AppState {
    rng: ThreadRng,
    id_box: usize,
    pub mutations: usize,
    pub children_born: usize,
    pub feeders_eaten: usize,
    pub kills: usize,
    pub rounds: usize,
}

impl AppState {
    pub fn new(id_start: usize) -> AppState {
        AppState {
            rng: rand::thread_rng(),
            id_box: id_start,
            mutations: 0,
            children_born: 0,
            feeders_eaten: 0,
            kills: 0,
            rounds: 0,
        }
    }
    pub fn rand<T: Rand>(&mut self) -> T {
        self.rng.gen()
    }
    pub fn rand_range<T: PartialOrd + SampleRange>(
        &mut self, low: T, high: T) -> T {
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
    pub fn next_creature_id(&mut self) -> usize {
        self.id_box += 1;
        self.id_box
    }
    pub fn rand_weighted_bool(&mut self, n: u32) -> bool {
        self.rng.gen_weighted_bool(n)
    }

}
