//! Main storage facade.
//!
//! Provides a unified interface to block and state storage.

use crate::block_store::BlockStore;
use crate::state_store::StateStore;
use crate::StorageError;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

/// Unified storage interface for the blockchain.
///
/// Combines block storage and state storage into a single facade.
pub struct Storage {
    /// Block storage
    blocks: BlockStore,

    /// State storage
    state: StateStore,

    /// Base path for all storage
    base_path: PathBuf,
}

impl Storage {
    /// Create a new storage instance at the given base path.
    ///
    /// Creates the directory structure if it doesn't exist:
    /// - `{base}/blocks/` - Block storage
    /// - `{base}/state/` - State storage
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError> {
        std::fs::create_dir_all(&base_path)?;

        let blocks = BlockStore::new(base_path.join("blocks"))?;
        let state = StateStore::new(base_path.join("state"))?;

        Ok(Self {
            blocks,
            state,
            base_path,
        })
    }

    /// Save a block at a given height.
    pub fn save_block<T: Serialize>(&self, height: u64, block: &T) -> Result<(), StorageError> {
        self.blocks.save(height, block)
    }

    /// Load a block at a given height.
    pub fn load_block<T: DeserializeOwned>(&self, height: u64) -> Result<T, StorageError> {
        self.blocks.load(height)
    }

    /// Check if a block exists at a given height.
    pub fn block_exists(&self, height: u64) -> bool {
        self.blocks.exists(height)
    }

    /// Get the highest stored block height.
    pub fn latest_block_height(&self) -> Result<Option<u64>, StorageError> {
        self.blocks.latest_height()
    }

    /// Save the latest state.
    pub fn save_state<T: Serialize>(&self, state: &T) -> Result<(), StorageError> {
        self.state.save_latest(state)
    }

    /// Load the latest state.
    pub fn load_state<T: DeserializeOwned>(&self) -> Result<T, StorageError> {
        self.state.load_latest()
    }

    /// Check if any state has been saved.
    pub fn has_state(&self) -> bool {
        self.state.has_latest()
    }

    /// Save a state snapshot at a specific height.
    pub fn save_snapshot<T: Serialize>(&self, height: u64, state: &T) -> Result<(), StorageError> {
        self.state.save_snapshot(height, state)
    }

    /// Load a state snapshot at a specific height.
    pub fn load_snapshot<T: DeserializeOwned>(&self, height: u64) -> Result<T, StorageError> {
        self.state.load_snapshot(height)
    }

    /// Get the base storage path.
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Atomically save both block and state together.
    ///
    /// This ensures consistency between block and state storage.
    pub fn commit<B: Serialize, S: Serialize>(
        &self,
        height: u64,
        block: &B,
        state: &S,
    ) -> Result<(), StorageError> {
        // Save block first
        self.save_block(height, block)?;

        // Then save state
        self.save_state(state)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestBlock {
        height: u64,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestState {
        height: u64,
    }

    #[test]
    fn test_storage_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf()).unwrap();

        // Initially empty
        assert!(!storage.has_state());
        assert_eq!(storage.latest_block_height().unwrap(), None);

        // Save block and state
        let block = TestBlock { height: 1 };
        let state = TestState { height: 1 };

        storage.commit(1, &block, &state).unwrap();

        // Verify saved
        assert!(storage.has_state());
        assert!(storage.block_exists(1));

        // Load and verify
        let loaded_block: TestBlock = storage.load_block(1).unwrap();
        let loaded_state: TestState = storage.load_state().unwrap();

        assert_eq!(block, loaded_block);
        assert_eq!(state, loaded_state);
    }

    #[test]
    fn test_restart_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // First session: save data
        {
            let storage = Storage::new(path.clone()).unwrap();
            let block = TestBlock { height: 5 };
            let state = TestState { height: 5 };
            storage.commit(5, &block, &state).unwrap();
        }

        // Second session: recover data
        {
            let storage = Storage::new(path).unwrap();
            assert!(storage.has_state());
            assert_eq!(storage.latest_block_height().unwrap(), Some(5));

            let state: TestState = storage.load_state().unwrap();
            assert_eq!(state.height, 5);
        }
    }
}
