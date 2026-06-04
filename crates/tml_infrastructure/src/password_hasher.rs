use argon2::{
    Argon2, PasswordHash, PasswordHasher as _, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

#[derive(Clone)]
pub struct PasswordHasher;

impl tml_application::app_trait::password_hasher::Trait for PasswordHasher {
    fn hash_password(
        &self,
        password: &str,
    ) -> Result<String, tml_application::app_trait::password_hasher::Error> {
        let salt = SaltString::generate(&mut OsRng);

        // Configurate Argon2id and use default arguments(m, t, p).
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                tml_application::app_trait::password_hasher::Error::HashingError(e.to_string())
            })?;

        Ok(password_hash.to_string())
    }

    fn verify_password(
        &self,
        password_input: &str,
        stored_hash_string: &str,
    ) -> Result<(), tml_application::app_trait::password_hasher::Error> {
        // Parse the PHC format string.
        // Algorithm, arguments(m, t, p) and salt are all included in the PHC string.
        let parsed_hash = PasswordHash::new(stored_hash_string).map_err(|e| {
            tml_application::app_trait::password_hasher::Error::HashingError(e.to_string())
        })?;

        // Verify the input password.
        // Argon2::default() will be covered by parsed_hash.
        match Argon2::default().verify_password(password_input.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(()),
            Err(argon2::password_hash::Error::Password) => {
                Err(tml_application::app_trait::password_hasher::Error::InvalidPassword)
            }
            Err(e) => {
                return Err(
                    tml_application::app_trait::password_hasher::Error::HashingError(e.to_string()),
                );
            }
        }
    }
}
