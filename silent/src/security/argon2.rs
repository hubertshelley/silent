use crate::{Result, SilentError, StatusCode};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn make_password(password: String) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            SilentError::business_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("make password failed: {e}"),
            )
        })?
        .to_string())
}

pub fn verify_password(password_hash: String, password: String) -> Result<bool> {
    let parsed_hash = PasswordHash::new(&password_hash).map_err(|e| {
        SilentError::business_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read password hash failed: {e}"),
        )
    })?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_test() {
        let password = "hello_password".to_string();
        let password_hash = make_password(password.clone()).unwrap();
        println!("{}", password_hash);
        assert!(verify_password(password_hash, password,).is_ok())
    }
}
