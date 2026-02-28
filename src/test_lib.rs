// Sprint 28 Mock External Struct Logic
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn normalize_vector(v: Vector3) -> Vector3 {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if length > 0.0 {
        Vector3 {
            x: v.x / length,
            y: v.y / length,
            z: v.z / length,
        }
    } else {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

pub fn calculate_hash(data: String) -> i64 {
    let mut hash: i64 = 0;
    for (i, b) in data.bytes().enumerate() {
        hash = hash.wrapping_add((b as i64) * (i as i64 + 1));
    }
    hash
}

pub fn greet_user(name: String) -> String {
    format!("Hello from Rust, {}!", name)
}
