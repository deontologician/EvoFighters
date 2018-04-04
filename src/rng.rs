use rand::{Rand, Rng, SeedableRng, XorShiftRng};
use rand::distributions;
use rand::distributions::range::SampleRange;
use creatures::{Creature, CreatureID};

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
        RngState::new(a_p, b_p, a.hash(), b.hash())
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

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        self.rng.shuffle(values)
    }
}
