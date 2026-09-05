pub fn len_of(text: &String) -> usize {
    text.len()
}

pub fn bump(value: &mut i32) -> i32 {
    *value += 1;
    *value
}

pub fn owned_len(text: String) -> usize {
    text.len()
}

pub fn reborrow<'a>(slot: &'a mut String) -> &'a str {
    slot.as_str()
}

pub fn main() {
    let name = String::from("ada");
    println!("{}", len_of(&name));
    let mut n = 41;
    println!("{}", bump(&mut n));
}
