//! State storage operations.
//!
//! Handles persistent storage of blockchain state with crash-safe writes.

use crate::StorageError;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

/// State storage manager.
pub struct StateStore {
    base_path: PathBuf,
}

impl StateStore {
    /// Create a new state store at the given path.
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    /// Get the path for the latest state file.
    fn latest_path(&self) -> PathBuf {
        self.base_path.join("latest.state")
    }

    /// Get the path for a temporary write file.
    fn temp_path(&self) -> PathBuf {
        self.base_path.join("latest.state.tmp")
    }

    /// Get the path for a state snapshot at a given height.
    fn snapshot_path(&self, height: u64) -> PathBuf {
        self.base_path.join(format!("snapshot_{:06}.state", height))
    }

    /// Save the latest state with crash-safe atomic write.
    pub fn save_latest<T: Serialize>(&self, state: &T) -> Result<(), StorageError> {
        let temp_path = self.temp_path();
        let final_path = self.latest_path();

        let bytes = bincode::serialize(state).map_err(|e| StorageError::Bincode { reason: e.to_string() })?;

        // Write to temp file
        fs::write(&temp_path, &bytes)?;

        // Atomic rename
        fs::rename(&temp_path, &final_path)?;

        Ok(())
    }

    /// Load the latest state.
    pub fn load_latest<T: DeserializeOwned>(&self) -> Result<T, StorageError> {
        let path = self.latest_path();

        if !path.exists() {
            return Err(StorageError::NotFound {
                key: "latest_state".to_string(),
            });
        }

        let bytes = fs::read(&path)?;
        bincode::deserialize(&bytes).map_err(|e| StorageError::Bincode { reason: e.to_string() })
    }

    /// Check if latest state exists.
    pub fn has_latest(&self) -> bool {
        self.latest_path().exists()
    }

    /// Save a state snapshot at a specific height.
    pub fn save_snapshot<T: Serialize>(&self, height: u64, state: &T) -> Result<(), StorageError> {
        let path = self.snapshot_path(height);
        let temp_path = self.base_path.join(format!("snapshot_{:06}.state.tmp", height));

        let bytes = bincode::serialize(state).map_err(|e| StorageError::Bincode { reason: e.to_string() })?;

        fs::write(&temp_path, &bytes)?;
        fs::rename(&temp_path, &path)?;

        Ok(())
    }

    /// Load a state snapshot at a specific height.
    pub fn load_snapshot<T: DeserializeOwned>(&self, height: u64) -> Result<T, StorageError> {
        let path = self.snapshot_path(height);

        if !path.exists() {
            return Err(StorageError::NotFound {
                key: format!("snapshot:{}", height),
            });
        }

        let bytes = fs::read(&path)?;
        bincode::deserialize(&bytes).map_err(|e| StorageError::Bincode { reason: e.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestState {
        height: u64,
        value: u64,
    }

    #[test]
    fn test_save_and_load_latest() {
        let temp_dir = TempDir::new().unwrap();
        let store = StateStore::new(temp_dir.path().to_path_buf()).unwrap();

        let state = TestState {
            height: 10,
            value: 42,
        };

        store.save_latest(&state).unwrap();
        let loaded: TestState = store.load_latest().unwrap();

        assert_eq!(state, loaded);
    }

    #[test]
    fn test_state_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let store = StateStore::new(temp_dir.path().to_path_buf()).unwrap();

        let result: Result<TestState, _> = store.load_latest();
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let store = StateStore::new(temp_dir.path().to_path_buf()).unwrap();

        let state = TestState {
            height: 100,
            value: 999,
        };

        store.save_snapshot(100, &state).unwrap();
        let loaded: TestState = store.load_snapshot(100).unwrap();

        assert_eq!(state, loaded);
    }
}
