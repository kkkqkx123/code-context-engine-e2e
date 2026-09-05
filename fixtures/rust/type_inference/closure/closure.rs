pub fn apply_twice(f: impl Fn(i32) -> i32, value: i32) -> i32 {
    f(f(value))
}

pub fn run() -> i32 {
    let double = |x: i32| x * 2;
    let items = vec![1, 2, 3];
    let incremented: Vec<i32> = items.iter().map(|x| x + 1).collect();
    let _ = incremented;
    apply_twice(double, 21)
}

pub fn main() {
    println!("{}", run());
}
