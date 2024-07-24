use rand::distributions::Alphanumeric;
use rand::Rng;

pub mod markdown;
pub mod notion;
pub mod settings;

fn generate_random_string(length: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
