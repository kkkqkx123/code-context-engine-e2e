use std::fmt::Display;

pub fn print_pair<T: Display, U: Display>(a: T, b: U) -> String {
    format!("{}: {}", a, b)
}

pub struct Container<T> {
    pub value: T,
}

impl<T: Clone> Container<T> {
    pub fn duplicate(&self) -> Container<T> {
        Container {
            value: self.value.clone(),
        }
    }
}

pub fn nested_generic() -> Vec<Option<i32>> {
    vec![Some(1), None, Some(3)]
}

pub fn identity<T>(x: T) -> T {
    x
}

pub fn wrap_in_vec<T>(item: T) -> Vec<T> {
    vec![item]
}

pub struct Pair<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> Pair<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Pair { first, second }
    }

    pub fn swap(self) -> Pair<B, A> {
        Pair {
            first: self.second,
            second: self.first,
        }
    }
}

pub fn collect_to_map<K, V, I>(iter: I) -> std::collections::HashMap<K, V>
where
    K: std::hash::Hash + Eq,
    I: IntoIterator<Item = (K, V)>,
{
    iter.into_iter().collect()
}

pub fn main() {
    let pair = print_pair(42, "hello");
    let container = Container { value: String::from("test") };
    let dup = container.duplicate();
    let nested = nested_generic();
    let wrapped = wrap_in_vec(10);
    let p = Pair::new(1, "one");
    let swapped = p.swap();
}
