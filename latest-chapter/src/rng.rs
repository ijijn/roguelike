use rltk::prelude::*;
use std::sync::{LazyLock, Mutex};

static RNG: LazyLock<Mutex<RandomNumberGenerator>> =
    LazyLock::new(|| Mutex::new(RandomNumberGenerator::new()));

pub fn reseed(seed: u64) {
    *RNG.lock().unwrap() = RandomNumberGenerator::seeded(seed);
}

#[must_use]
pub fn roll_dice(n: i32, die_type: i32) -> i32 {
    RNG.lock().unwrap().roll_dice(n, die_type)
}

#[must_use]
pub fn range(min: i32, max: i32) -> i32 {
    RNG.lock().unwrap().range(min, max)
}
