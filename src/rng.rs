use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

//ChaChaRng pseudo-random number generator
pub fn generator_random_number() -> u64 {
    let mut r = ChaChaRng::from_entropy();
    ChaChaRng::next_u64(&mut r)
}
