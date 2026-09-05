pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn with_count(count: i32) -> Self {
        Self { count }
    }

    pub fn increment(&mut self) -> &mut Self {
        self.count += 1;
        self
    }

    pub fn value(&self) -> i32 {
        self.count
    }
}

pub trait Summable {
    type Item;

    fn sum(items: &[Self::Item]) -> Self::Item;
}

pub struct IntSum;

impl Summable for IntSum {
    type Item = i32;

    fn sum(items: &[Self::Item]) -> Self::Item {
        items.iter().sum()
    }
}

pub fn main() {
    let mut counter = Counter::new();
    counter.increment();
    println!("{}", counter.value());
}
