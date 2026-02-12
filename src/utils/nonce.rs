use rand::Rng;

/// Generate a random nonce as a u32.
pub fn generate_nonce() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>()
}
