use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn generate_random_id(seed: &AtomicU64) -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut s = DefaultHasher::new();
    seed.fetch_add(1, Ordering::Relaxed);
    let feed = seed.load(Ordering::Relaxed) as u128 + nanos;
    feed.hash(&mut s);
    s.finish()
}