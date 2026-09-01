pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}
