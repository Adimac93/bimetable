use argon2::hash_encoded;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use secrecy::{ExposeSecret, SecretString};
use validator::{ValidationErrors, Validate, ValidationError};

use super::models::ValidatedUserData;

pub fn hash_pass(pass: SecretString) -> Result<String, argon2::Error> {
    hash_encoded(
        pass.expose_secret().as_bytes(),
        random_salt().as_bytes(),
        &argon2::Config::default(),
    )
}

pub fn random_salt() -> String {
    let mut rng = thread_rng();
    (0..8).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn pass_is_strong(user_password: &str, user_inputs: &[&str]) -> bool {
    let score = zxcvbn::zxcvbn(user_password, user_inputs);
    score.map_or(false, |entropy| entropy.score() >= 3)
}

pub fn validate_usernames(login: &str, username: &str) -> Result<(), ValidationErrors> {
    ValidatedUserData {
        login: login.to_string(),
        username: username.to_string(),
    }.validate()
}

pub fn is_ascii_or_latin_extended(text: &str) -> Result<(), ValidationError> {
    if text.chars().all(|x| x as u32 <= 687) { Ok(()) }
    else { Err(ValidationError::new("Non-ASCII and non-latin-extended characters detected")) }
}
