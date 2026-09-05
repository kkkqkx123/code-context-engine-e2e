pub fn first<'a>(items: &'a [String]) -> &'a str {
    &items[0]
}

pub fn append_copy(values: &mut Vec<String>, extra: &str) {
    values.push(extra.to_string());
}

pub fn owned_or_borrowed(flag: bool) -> String {
    let owned = String::from("owned");
    let borrowed: &str = &owned;
    if flag {
        owned
    } else {
        borrowed.to_string()
    }
}

pub fn collect_refs<'a>(a: &'a str, b: &'a str) -> Vec<&'a str> {
    vec![a, b]
}

pub fn main() {
    let items = vec![String::from("a")];
    println!("{}", first(&items));
}
