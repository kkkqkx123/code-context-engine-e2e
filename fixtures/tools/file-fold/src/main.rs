use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub roles: Vec<String>,
}

pub struct UserService {
    users: HashMap<u64, User>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn create_user(&mut self, id: u64, name: &str) -> User {
        let user = User {
            id,
            name: name.to_string(),
            roles: vec!["member".to_string()],
        };
        self.users.insert(id, user.clone());
        user
    }

    pub fn find_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn grant_role(&mut self, id: u64, role: &str) -> Option<User> {
        let user = self.users.get_mut(&id)?;
        if !user.roles.iter().any(|existing| existing == role) {
            user.roles.push(role.to_string());
        }
        Some(user.clone())
    }

    fn normalize(name: &str) -> String {
        name.trim().replace(' ', "_").to_lowercase()
    }
}

pub fn format_user(user: &User) -> String {
    format!("{}#{}:{}", UserService::normalize(&user.name), user.id, user.roles.join("+"))
}

pub fn summarize_users(users: &[User]) -> String {
    users
        .iter()
        .map(|user| format_user(user))
        .collect::<Vec<_>>()
        .join(", ")
}
