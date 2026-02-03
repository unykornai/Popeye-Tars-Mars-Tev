//! Consensus state persistence.
//!
//! Stores consensus artifacts for crash recovery:
//! - Round state
//! - Vote sets
//! - Finality certificates
//!
//! All writes are crash-safe (atomic via temp file + rename).

use crate::StorageError;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persists consensus state for crash recovery.
pub struct ConsensusStore {
    /// Directory for consensus data.
    base_path: PathBuf,
}

impl ConsensusStore {
    /// Create a new consensus store.
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    /// Atomically write data to a file.
    fn atomic_write(&self, path: &PathBuf, data: &[u8]) -> Result<(), StorageError> {
        let temp_path = path.with_extension("tmp");

        // Write to temp file and sync
        {
            let mut file = fs::File::create(&temp_path)?;
            std::io::Write::write_all(&mut file, data)?;
            file.sync_all()?;
        } // File closed here

        // Atomic rename
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Save round state for recovery.
    pub fn save_round_state<T: Serialize>(&self, state: &T) -> Result<(), StorageError> {
        let path = self.base_path.join("round_state.json");
        let data = serde_json::to_vec_pretty(state)?;
        self.atomic_write(&path, &data)
    }

    /// Load round state.
    pub fn load_round_state<T: DeserializeOwned>(&self) -> Result<Option<T>, StorageError> {
        let path = self.base_path.join("round_state.json");

        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read(&path)?;
        let state = serde_json::from_slice(&data)?;
        Ok(Some(state))
    }

    /// Save a finality certificate.
    pub fn save_finality_certificate<T: Serialize>(
        &self,
        height: u64,
        cert: &T,
    ) -> Result<(), StorageError> {
        let path = self
            .base_path
            .join(format!("finality_{:08}.json", height));
        let data = serde_json::to_vec_pretty(cert)?;
        self.atomic_write(&path, &data)
    }

    /// Load a finality certificate.
    pub fn load_finality_certificate<T: DeserializeOwned>(
        &self,
        height: u64,
    ) -> Result<Option<T>, StorageError> {
        let path = self
            .base_path
            .join(format!("finality_{:08}.json", height));

        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read(&path)?;
        let cert = serde_json::from_slice(&data)?;
        Ok(Some(cert))
    }

    /// Get the highest finalized height.
    pub fn latest_finalized_height(&self) -> Result<Option<u64>, StorageError> {
        let mut max_height: Option<u64> = None;

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("finality_") && name_str.ends_with(".json") {
                // Parse height from filename: finality_00000001.json
                if let Some(height_str) = name_str
                    .strip_prefix("finality_")
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    if let Ok(height) = height_str.parse::<u64>() {
                        max_height = Some(max_height.map_or(height, |m| m.max(height)));
                    }
                }
            }
        }

        Ok(max_height)
    }

    /// Save the validator set.
    pub fn save_validator_set<T: Serialize>(&self, set: &T) -> Result<(), StorageError> {
        let path = self.base_path.join("validators.json");
        let data = serde_json::to_vec_pretty(set)?;
        self.atomic_write(&path, &data)
    }

    /// Load the validator set.
    pub fn load_validator_set<T: DeserializeOwned>(&self) -> Result<Option<T>, StorageError> {
        let path = self.base_path.join("validators.json");

        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read(&path)?;
        let set = serde_json::from_slice(&data)?;
        Ok(Some(set))
    }

    /// Check if we have any consensus state.
    pub fn has_state(&self) -> bool {
        self.base_path.join("round_state.json").exists()
    }

    /// Clear all consensus state (for testing/reset).
    pub fn clear(&self) -> Result<(), StorageError> {
        if self.base_path.exists() {
            for entry in fs::read_dir(&self.base_path)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "json") {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestRoundState {
        height: u64,
        round: u64,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestCert {
        height: u64,
        block_hash: [u8; 32],
    }

    #[test]
    fn round_state_persistence() {
        let temp = TempDir::new().unwrap();
        let store = ConsensusStore::new(temp.path().to_path_buf()).unwrap();

        let state = TestRoundState { height: 5, round: 2 };
        store.save_round_state(&state).unwrap();

        let loaded: Option<TestRoundState> = store.load_round_state().unwrap();
        assert_eq!(loaded, Some(state));
    }

    #[test]
    fn finality_certificate_persistence() {
        let temp = TempDir::new().unwrap();
        let store = ConsensusStore::new(temp.path().to_path_buf()).unwrap();

        let cert1 = TestCert {
            height: 1,
            block_hash: [1u8; 32],
        };
        let cert5 = TestCert {
            height: 5,
            block_hash: [5u8; 32],
        };

        store.save_finality_certificate(1, &cert1).unwrap();
        store.save_finality_certificate(5, &cert5).unwrap();

        let loaded1: Option<TestCert> = store.load_finality_certificate(1).unwrap();
        let loaded5: Option<TestCert> = store.load_finality_certificate(5).unwrap();

        assert_eq!(loaded1, Some(cert1));
        assert_eq!(loaded5, Some(cert5));

        // Highest finalized
        let latest = store.latest_finalized_height().unwrap();
        assert_eq!(latest, Some(5));
    }

    #[test]
    fn recovery_after_simulated_crash() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().to_path_buf();

        // Session 1: save state
        {
            let store = ConsensusStore::new(path.clone()).unwrap();
            store
                .save_round_state(&TestRoundState { height: 10, round: 3 })
                .unwrap();
            store
                .save_finality_certificate(
                    9,
                    &TestCert {
                        height: 9,
                        block_hash: [9u8; 32],
                    },
                )
                .unwrap();
        }

        // Session 2: recover
        {
            let store = ConsensusStore::new(path).unwrap();
            assert!(store.has_state());

            let state: TestRoundState = store.load_round_state().unwrap().unwrap();
            assert_eq!(state.height, 10);
            assert_eq!(state.round, 3);

            let latest = store.latest_finalized_height().unwrap();
            assert_eq!(latest, Some(9));
        }
    }
}
