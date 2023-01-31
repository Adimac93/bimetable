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

#[derive(Serialize, Deserialize, Validate)]
pub struct RegisterCredentials {
    #[validate(non_control_character, custom = "is_ascii_or_latin_extended", does_not_contain = " ", length(min = 4, max = 20))]
    pub login: String,
    pub password: String,
    #[validate(non_control_character, custom = "is_ascii_or_latin_extended", length(min = 4, max = 20))]
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

fn is_ascii_or_latin_extended(text: &str) -> Result<(), ValidationError> {
    if text.chars().all(|x| x as u32 <= 687) { Ok(()) }
    else { Err(ValidationError::new("Non-ASCII and non-latin-extended characters detected")) }
}