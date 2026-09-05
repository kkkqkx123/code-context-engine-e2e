use crate::{format_name, Greeter};

pub fn render(name: &str) -> String {
    let greeter = Greeter;
    format!("{}|{}", greeter.greet(name), format_name(name))
}
