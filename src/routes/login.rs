use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Login {
    pub login: String,
    password: String,
}

impl Login {
    pub fn check_password(&self, password: &String) -> bool {
        self.password == *password
    }
}