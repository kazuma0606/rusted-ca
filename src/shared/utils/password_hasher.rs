// shared/utils/password_hasher.rs
// パスワードハッシュ化・検証ユーティリティ
// 2025/7/13

use crate::shared::error::infrastructure_error::{InfrastructureError, PasswordHasherResult};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// パスワードをハッシュ化（エンコード）する
pub fn encode(plain: &str) -> PasswordHasherResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| InfrastructureError::PasswordHashing {
            message: e.to_string(),
        })
}

/// 平文パスワードとハッシュ値を照合（デコード/検証）する
pub fn decode(plain: &str, hash: &str) -> PasswordHasherResult<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| InfrastructureError::PasswordHashing {
            message: e.to_string(),
        })?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(plain.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_success() {
        let password = "SuperSecret123!";
        let hash = encode(password).expect("Hashing should succeed");
        assert!(decode(password, &hash).expect("Verification should succeed"));
    }

    #[test]
    fn test_decode_fail_with_wrong_password() {
        let password = "SuperSecret123!";
        let wrong_password = "WrongPassword456!";
        let hash = encode(password).expect("Hashing should succeed");
        assert!(!decode(wrong_password, &hash).expect("Verification should succeed"));
    }

    #[test]
    fn test_decode_fail_with_invalid_hash() {
        let password = "SuperSecret123!";
        let invalid_hash = "$argon2id$v=19$m=4096,t=3,p=1$invalidsalt$invalidhash";
        let result = decode(password, invalid_hash);
        assert!(result.is_err());
    }
}
