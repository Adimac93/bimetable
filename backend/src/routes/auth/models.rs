use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Serialize, Deserialize)]
pub struct LoginCredentials {
    pub login: String,
    pub password: String,
}

impl LoginCredentials {
    pub fn new(login: &str, password: &str) -> Self {
        Self {
            login: login.into(),
            password: password.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RegisterCredentials {
    pub login: String,
    pub password: String,
    pub username: String,
}

impl RegisterCredentials {
    pub fn new(login: &str, password: &str, username: &str) -> Self {
        Self {
            login: login.into(),
            password: password.into(),
            username: username.into(),
        }
    }
}
