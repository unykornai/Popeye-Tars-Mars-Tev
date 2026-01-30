//! # TEV — Trusted Execution & Validation
//!
//! TEV is the cryptographic firewall for Unykorn L1.
//! It validates signatures and enforces the transport format.
//!
//! ## Trust Model
//!
//! Nothing crosses from POPEYE to MARS without passing TEV.
//! This separation ensures:
//! - Network spam cannot corrupt state
//! - Invalid blocks cannot cause forks
//! - Malformed transactions cannot crash the runtime
//!
//! ## Design Principles
//!
//! - **Stateless**: No storage, no persistence
//! - **Pure**: Verification only, no side effects
//! - **Type-safe**: Verified vs Unverified types

pub mod error;
pub mod signature;
pub mod verified;

pub use error::ValidationError;
pub use signature::{sign_message, verify_signature, Keypair};
pub use verified::{VerifiedBlock, VerifiedTransaction};

/// Verify a raw transaction payload.
///
/// # Format
///
/// The payload must be at least 96 bytes:
/// - Last 64 bytes: Ed25519 signature
/// - Preceding 32 bytes: Public key (signer)
/// - Remaining bytes: Transaction data
///
/// # Returns
///
/// A `VerifiedTransaction` that can be safely passed to MARS.
pub fn verify_transaction(payload: &[u8]) -> Result<VerifiedTransaction, ValidationError> {
    if payload.len() < 96 {
        return Err(ValidationError::InvalidFormat {
            reason: format!("payload too short: {} bytes, minimum 96", payload.len()),
        });
    }

    let sig_start = payload.len() - 64;
    let pubkey_start = sig_start - 32;

    let data = &payload[..pubkey_start];
    let pubkey_bytes = &payload[pubkey_start..sig_start];
    let signature_bytes = &payload[sig_start..];

    // Convert to fixed arrays
    let pubkey: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| ValidationError::InvalidFormat {
            reason: "invalid public key length".to_string(),
        })?;

    let signature: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| ValidationError::InvalidFormat {
            reason: "invalid signature length".to_string(),
        })?;

    // Verify the signature
    verify_signature(&pubkey, data, &signature)?;

    Ok(VerifiedTransaction {
        data: data.to_vec(),
        signer: pubkey,
        signature,
    })
}

/// Verify a raw block payload.
///
/// Similar format to transactions but for block data.
pub fn verify_block(payload: &[u8]) -> Result<VerifiedBlock, ValidationError> {
    if payload.len() < 96 {
        return Err(ValidationError::InvalidFormat {
            reason: format!("block payload too short: {} bytes", payload.len()),
        });
    }

    let sig_start = payload.len() - 64;
    let pubkey_start = sig_start - 32;

    let data = &payload[..pubkey_start];
    let pubkey_bytes = &payload[pubkey_start..sig_start];
    let signature_bytes = &payload[sig_start..];

    let producer: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| ValidationError::InvalidFormat {
            reason: "invalid producer key length".to_string(),
        })?;

    let signature: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| ValidationError::InvalidFormat {
            reason: "invalid block signature length".to_string(),
        })?;

    verify_signature(&producer, data, &signature)?;

    Ok(VerifiedBlock {
        data: data.to_vec(),
        producer,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_short_payload() {
        let short = vec![0u8; 50];
        assert!(verify_transaction(&short).is_err());
    }

    #[test]
    fn test_verify_valid_transaction() {
        let keypair = Keypair::generate();
        let data = b"test transaction data";

        let mut payload = Vec::new();
        payload.extend_from_slice(data);
        payload.extend_from_slice(&keypair.public_key());

        let signature = keypair.sign(data);
        payload.extend_from_slice(&signature);

        let result = verify_transaction(&payload);
        assert!(result.is_ok());

        let verified = result.unwrap();
        assert_eq!(verified.signer, keypair.public_key());
        assert_eq!(verified.data, data);
    }

    #[test]
    fn test_reject_invalid_signature() {
        let keypair = Keypair::generate();
        let data = b"test transaction data";

        let mut payload = Vec::new();
        payload.extend_from_slice(data);
        payload.extend_from_slice(&keypair.public_key());
        payload.extend_from_slice(&[0u8; 64]); // Invalid signature

        let result = verify_transaction(&payload);
        assert!(result.is_err());
    }
}
