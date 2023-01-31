use anyhow::anyhow;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand;

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
