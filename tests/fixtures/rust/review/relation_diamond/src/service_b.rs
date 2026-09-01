use crate::repository;

pub fn print_summary(user_id: u32) {
    let user = repository::get_user(user_id);
    println!("User: {} (id={})", user.0, user.1);
}
