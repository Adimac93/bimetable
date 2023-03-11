use anyhow::anyhow;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::collections::HashSet;
use validator::{Validate, ValidationError, ValidationErrors};

use super::models::ValidatedUserData;

pub fn hash_pass(password: String) -> anyhow::Result<String> {
    let salt = SaltString::generate(rand::thread_rng());
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e).context("failed to hash password"))?
        .to_string())
}

pub fn verify_pass(password: String, hash: String) -> anyhow::Result<bool> {
    let hash = PasswordHash::new(&hash).map_err(|e| anyhow!(e).context("password hash invalid"))?;
    let res = Argon2::default().verify_password(password.as_bytes(), &hash);
    match res {
        Ok(()) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow!(e).context("failed to verify password")),
    }
}

pub fn pass_is_strong(user_password: &str, user_inputs: &[&str]) -> bool {
    let score = zxcvbn::zxcvbn(user_password, user_inputs);
    score.map_or(false, |entropy| entropy.score() >= 3)
}

pub fn validate_usernames(login: &str, username: &str) -> Result<(), ValidationErrors> {
    ValidatedUserData {
        login: login.to_string(),
        username: username.to_string(),
    }
    .validate()
}

pub fn is_ascii_or_latin_extended(text: &str) -> Result<(), ValidationError> {
    if text.chars().all(|x| x as u32 <= 687) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Non-ASCII and non-latin-extended characters detected",
        ))
    }
}

pub fn random_username_tag(used_tags: HashSet<i32>) -> Option<i32> {
    let mut rng = thread_rng();
    (0..10000)
        .filter(|x| !used_tags.contains(x))
        .choose(&mut rng)
}

#[test]
fn random_username_tag_overflow() {
    let set = HashSet::<i32>::from_iter(0..10000);
    let res = random_username_tag(set);

    assert_eq!(None, res)
}

#[test]
fn random_username_tag_not_overflowing() {
    let set = HashSet::<i32>::from_iter(1..10000);
    let res = random_username_tag(set);

    assert!(res.is_some())
}
