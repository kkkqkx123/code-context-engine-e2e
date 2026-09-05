pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn destructure_tuple(pair: (i32, String)) -> String {
    let (num, text) = pair;
    let _ = num;
    text
}

pub fn destructure_struct(p: Point) -> i32 {
    let Point { x, y } = p;
    x + y
}

pub fn destructure_at(value: Option<String>) -> String {
    match value {
        Some(text @ _) => text,
        None => String::from("none"),
    }
}

pub fn destructure_if_let(expr: Option<(i32, i32)>) -> i32 {
    if let Some((a, b)) = expr {
        a + b
    } else {
        0
    }
}

pub fn main() {
    println!("{}", destructure_tuple((1, String::from("one"))));
    println!("{}", destructure_struct(Point { x: 1, y: 2 }));
}
