use crate::model::User;

pub fn normalize_name(input: &str) -> String {
    input.trim().to_lowercase()
}

pub fn format_user(user: &User) -> String {
    user.display_name()
}
