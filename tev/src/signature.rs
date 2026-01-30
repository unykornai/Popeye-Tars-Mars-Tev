//! Ed25519 signature operations.
//!
//! This module provides cryptographic signing and verification
//! using the Ed25519 signature scheme.

use crate::ValidationError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// A keypair for signing operations.
pub struct Keypair {
    signing_key: SigningKey,
}

impl Keypair {
    /// Generate a new random keypair.
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        Self { signing_key }
    }

    /// Create a keypair from a secret key (32 bytes).
    pub fn from_secret(secret: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(secret);
        Self { signing_key }
    }

    /// Get the public key (32 bytes).
    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Sign a message, returning a 64-byte signature.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature = self.signing_key.sign(message);
        signature.to_bytes()
    }
}

/// Verify an Ed25519 signature.
///
/// # Arguments
///
/// * `public_key` - The signer's 32-byte public key
/// * `message` - The message that was signed
/// * `signature` - The 64-byte signature
///
/// # Returns
///
/// `Ok(())` if the signature is valid, `Err(ValidationError)` otherwise.
pub fn verify_signature(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), ValidationError> {
    let verifying_key =
        VerifyingKey::from_bytes(public_key).map_err(|_| ValidationError::InvalidPublicKey)?;

    let sig = Signature::from_bytes(signature);

    verifying_key
        .verify(message, &sig)
        .map_err(|_| ValidationError::InvalidSignature)?;

    Ok(())
}

/// Sign a message with a secret key.
///
/// # Arguments
///
/// * `secret` - The 32-byte secret key
/// * `message` - The message to sign
///
/// # Returns
///
/// A 64-byte Ed25519 signature.
pub fn sign_message(secret: &[u8; 32], message: &[u8]) -> [u8; 64] {
    let keypair = Keypair::from_secret(secret);
    keypair.sign(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let keypair = Keypair::generate();
        let message = b"hello world";
        let signature = keypair.sign(message);

        let result = verify_signature(&keypair.public_key(), message, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_wrong_message() {
        let keypair = Keypair::generate();
        let signature = keypair.sign(b"original message");

        let result = verify_signature(&keypair.public_key(), b"wrong message", &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_wrong_key() {
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        let message = b"hello";
        let signature = keypair1.sign(message);

        let result = verify_signature(&keypair2.public_key(), message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_keypair_from_secret() {
        let secret = [42u8; 32];
        let keypair1 = Keypair::from_secret(&secret);
        let keypair2 = Keypair::from_secret(&secret);

        assert_eq!(keypair1.public_key(), keypair2.public_key());
    }
}
