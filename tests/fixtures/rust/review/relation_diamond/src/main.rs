mod service_a;
mod service_b;
mod repository;

fn main() {
    let id = service_a::create_user("alice");
    service_b::print_summary(id);
}
