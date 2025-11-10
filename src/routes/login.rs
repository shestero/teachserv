use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Login {
    pub login: String,
    password: String,
    token: Option<String>,
    captcha: Option<String>,
}

impl Login {
    pub fn check_password(&self, password: &String) -> bool {
        self.password == *password
    }

    pub fn check_captcha(&self) -> bool {
        self.captcha.is_some() && self.captcha == self.token // TODO
    }
}