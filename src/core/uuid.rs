use std::time::SystemTime;

pub fn generate_random_uuid() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards");

    format!("{:x}", timestamp.as_nanos() as u64)
}