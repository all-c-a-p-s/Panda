use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

//ChaChaRng pseudo-random number generator
pub fn random_u32() -> u32 {
    let mut r = ChaChaRng::from_entropy();
    ChaChaRng::next_u32(&mut r)
}

pub fn random_hash_u64() -> u64 {
    let mut r = ChaChaRng::from_entropy();
    ChaChaRng::next_u64(&mut r)
}

// method suggested by Tord Romstad (SF developer)
pub fn random_u64() -> u64 {
    // reduce non-zero bits in slices of 6
    let n1 = random_u32() as u64 & 0xFFFF;
    let n2 = random_u32() as u64 & 0xFFFF;
    let n3 = random_u32() as u64 & 0xFFFF;
    let n4 = random_u32() as u64 & 0xFFFF;

    n1 | n2 << 16 | n3 << 32 | n4 << 48
}

pub fn magic_candidate() -> u64 {
    // aim to generate random number with very few non-zero bits
    random_u64() & random_u64() & random_u64()
}

pub struct XorShiftU64 {
    pub state: u64,
}

const SEED: u64 = 0xF8D1C463A579BE02;

//need a rng that I can call in a const fn
impl XorShiftU64 {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self { state: SEED }
    }

    pub const fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}
