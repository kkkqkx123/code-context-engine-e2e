use crate::repository;

pub fn create_user(name: &str) -> u32 {
    let id = repository::next_id();
    repository::save_user(id, name);
    id
}
