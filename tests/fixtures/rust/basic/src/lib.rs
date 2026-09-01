pub fn process() -> i32 {
    let x = helper::internal();
    x * 2
}

mod helper {
    pub fn internal() -> i32 {
        21
    }
}

