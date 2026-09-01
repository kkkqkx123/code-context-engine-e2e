//! Original version of the file for file edit testing

pub fn original_function() -> i32 {
    42
}

pub struct OriginalStruct {
    pub field: String,
}

impl OriginalStruct {
    pub fn new() -> Self {
        Self {
            field: "original".to_string(),
        }
    }
}
