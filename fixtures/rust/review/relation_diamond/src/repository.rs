use std::collections::HashMap;

static mut NEXT_ID: u32 = 1;

pub fn next_id() -> u32 {
    unsafe {
        let id = NEXT_ID;
        NEXT_ID += 1;
        id
    }
}

pub fn save_user(id: u32, name: &str) {
    tracing::info!("save user {}: {}", id, name);
}

pub fn get_user(id: u32) -> (String, u32) {
    (format!("user_{}", id), id)
}
