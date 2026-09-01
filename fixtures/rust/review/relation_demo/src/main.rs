mod model;
mod repository;
mod utils;

fn main() {
    let _summary = repository::load_and_format_user(42);
}
