pub fn calculate_hash(data: String) -> i64 {
    // A simple mock hash function to test FFI bridging
    let mut hash: i64 = 0;
    for (i, b) in data.bytes().enumerate() {
        hash = hash.wrapping_add((b as i64) * (i as i64 + 1));
    }
    hash
}

pub fn greet_user(name: String) -> String {
    format!("Hello from Rust, {}!", name)
}
