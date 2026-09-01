//! Modified version of the file for file edit testing

pub fn modified_function() -> i32 {
    100
}

pub fn new_function() -> String {
    "new".to_string()
}

pub struct ModifiedStruct {
    pub field: String,
    pub new_field: i32,
}

impl ModifiedStruct {
    pub fn new() -> Self {
        Self {
            field: "modified".to_string(),
            new_field: 0,
        }
    }
    
    pub fn with_value(value: i32) -> Self {
        Self {
            field: "modified".to_string(),
            new_field: value,
        }
    }
}
