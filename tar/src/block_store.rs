//! Block storage operations.
//!
//! Handles persistent storage of blocks with crash-safe writes.

use crate::StorageError;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

/// Block storage manager.
pub struct BlockStore {
    base_path: PathBuf,
}

impl BlockStore {
    /// Create a new block store at the given path.
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    /// Get the path for a block at a given height.
    fn block_path(&self, height: u64) -> PathBuf {
        self.base_path.join(format!("{:06}.block", height))
    }

    /// Get the path for a temporary write file.
    fn temp_path(&self, height: u64) -> PathBuf {
        self.base_path.join(format!("{:06}.block.tmp", height))
    }

    /// Save a block with crash-safe atomic write.
    ///
    /// Uses write-to-temp + rename pattern to ensure atomicity.
    pub fn save<T: Serialize>(&self, height: u64, block: &T) -> Result<(), StorageError> {
        let temp_path = self.temp_path(height);
        let final_path = self.block_path(height);

        // Serialize
        let bytes = bincode::serialize(block).map_err(|_| StorageError::Serialization)?;

        // Write to temp file
        fs::write(&temp_path, &bytes)?;

        // Atomic rename
        fs::rename(&temp_path, &final_path)?;

        Ok(())
    }

    /// Load a block at a given height.
    pub fn load<T: DeserializeOwned>(&self, height: u64) -> Result<T, StorageError> {
        let path = self.block_path(height);

        if !path.exists() {
            return Err(StorageError::NotFound {
                key: format!("block:{}", height),
            });
        }

        let bytes = fs::read(&path)?;
        bincode::deserialize(&bytes).map_err(|_| StorageError::Serialization)
    }

    /// Check if a block exists at a given height.
    pub fn exists(&self, height: u64) -> bool {
        self.block_path(height).exists()
    }

    /// Get the highest stored block height.
    pub fn latest_height(&self) -> Result<Option<u64>, StorageError> {
        let mut highest: Option<u64> = None;

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.ends_with(".block") && !name_str.ends_with(".tmp") {
                if let Some(height_str) = name_str.strip_suffix(".block") {
                    if let Ok(height) = height_str.parse::<u64>() {
                        highest = Some(highest.map_or(height, |h| h.max(height)));
                    }
                }
            }
        }

        Ok(highest)
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
        data: String,
    }

    #[test]
    fn test_save_and_load_block() {
        let temp_dir = TempDir::new().unwrap();
        let store = BlockStore::new(temp_dir.path().to_path_buf()).unwrap();

        let block = TestBlock {
            height: 1,
            data: "test".to_string(),
        };

        store.save(1, &block).unwrap();
        let loaded: TestBlock = store.load(1).unwrap();

        assert_eq!(block, loaded);
    }

    #[test]
    fn test_block_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let store = BlockStore::new(temp_dir.path().to_path_buf()).unwrap();

        let result: Result<TestBlock, _> = store.load(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_latest_height() {
        let temp_dir = TempDir::new().unwrap();
        let store = BlockStore::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(store.latest_height().unwrap(), None);

        let block = TestBlock {
            height: 5,
            data: "test".to_string(),
        };
        store.save(5, &block).unwrap();

        assert_eq!(store.latest_height().unwrap(), Some(5));
    }
}
