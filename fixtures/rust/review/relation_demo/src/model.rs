use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: u32,
    name: String,
}

impl User {
    pub fn create(id: u32, name: impl Into<String>) -> Self {
        let name = name.into();
        Self { id, name }
    }

    pub fn display_name(&self) -> String {
        format!("{}#{}", self.name, self.id)
    }

    pub fn identifier(&self) -> u32 {
        self.id
    }
}
