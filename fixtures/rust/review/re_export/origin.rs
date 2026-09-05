pub fn format_name(name: &str) -> String {
    format!("hello {}", name)
}

pub struct Greeter;

impl Greeter {
    pub fn greet(&self, name: &str) -> String {
        format_name(name)
    }
}
