#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Password hashing error: {0}")]
    HashingError(String),
    #[error("Invalid password")]
    InvalidPassword,
}

pub trait Trait: Send + Sync + Clone {
    /// Hash the input password, return PHC format string.
    fn hash_password(&self, password: &str) -> Result<String, Error>;

    /// verify the password, check if it matches the hash_string.
    fn verify_password(&self, password_input: &str, stored_hash_string: &str) -> Result<(), Error>;
}
