use crate::model::User;
use crate::utils::{format_user, normalize_name};

pub fn load_user(id: u32) -> User {
    let name = normalize_name("  Ada Lovelace  ");
    User::create(id, name)
}

pub fn save_user(user: &User) -> u32 {
    user.identifier()
}

pub fn load_and_format_user(id: u32) -> String {
    let user = load_user(id);
    format_user(&user)
}
