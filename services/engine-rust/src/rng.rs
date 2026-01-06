use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn seeded_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

pub fn gen_range_f64(rng: &mut StdRng, min: f64, max: f64) -> f64 {
    rng.gen_range(min..max)
}

pub fn gen_range_usize(rng: &mut StdRng, upper: usize) -> usize {
    rng.gen_range(0..upper)
}
