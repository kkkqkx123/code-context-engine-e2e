pub fn handle_option(opt: Option<String>) -> String {
    if let Some(value) = opt {
        value
    } else {
        String::from("default")
    }
}

pub fn handle_result(result: Result<i32, String>) -> String {
    match result {
        Ok(val) => format!("value: {}", val),
        Err(e) => format!("error: {}", e),
    }
}

pub fn nested_match(opt: Option<Result<i32, String>>) -> String {
    match opt {
        Some(Ok(val)) => format!("ok: {}", val),
        Some(Err(e)) => format!("err: {}", e),
        None => "none".to_string(),
    }
}

pub fn if_let_chain(x: Option<String>, y: Option<i32>) -> String {
    if let Some(ref name) = x {
        if let Some(age) = y {
            format!("{} is {}", name, age)
        } else {
            format!("{} has no age", name)
        }
    } else {
        "unknown".to_string()
    }
}

pub fn main() {
    let opt = Some(String::from("hello"));
    let result: Result<i32, String> = Ok(42);
    println!("{}", handle_option(opt));
    println!("{}", handle_result(result));
    println!("{}", nested_match(Some(Ok(10))));
    println!("{}", if_let_chain(Some(String::from("Alice")), Some(30)));
}
