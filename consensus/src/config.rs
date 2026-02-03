//! Consensus engine configuration.

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for the consensus engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Timeout for proposal phase.
    #[serde(with = "humantime_serde")]
    pub propose_timeout: Duration,

    /// Timeout for prevote phase.
    #[serde(with = "humantime_serde")]
    pub prevote_timeout: Duration,

    /// Timeout for commit phase.
    #[serde(with = "humantime_serde")]
    pub commit_timeout: Duration,

    /// Base timeout increase per round (for exponential backoff).
    #[serde(with = "humantime_serde")]
    pub timeout_delta: Duration,

    /// Maximum rounds before giving up on a height.
    pub max_rounds: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            propose_timeout: Duration::from_secs(3),
            prevote_timeout: Duration::from_secs(2),
            commit_timeout: Duration::from_secs(2),
            timeout_delta: Duration::from_millis(500),
            max_rounds: 10,
        }
    }
}

impl ConsensusConfig {
    /// Calculate propose timeout for a specific round (exponential backoff).
    pub fn propose_timeout_for_round(&self, round: u64) -> Duration {
        self.propose_timeout + self.timeout_delta * round as u32
    }

    /// Calculate prevote timeout for a specific round.
    pub fn prevote_timeout_for_round(&self, round: u64) -> Duration {
        self.prevote_timeout + self.timeout_delta * round as u32
    }

    /// Calculate commit timeout for a specific round.
    pub fn commit_timeout_for_round(&self, round: u64) -> Duration {
        self.commit_timeout + self.timeout_delta * round as u32
    }
}

mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ms = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = ConsensusConfig::default();
        assert_eq!(config.propose_timeout, Duration::from_secs(3));
        assert_eq!(config.prevote_timeout, Duration::from_secs(2));
        assert_eq!(config.max_rounds, 10);
    }

    #[test]
    fn exponential_backoff() {
        let config = ConsensusConfig::default();

        let t0 = config.propose_timeout_for_round(0);
        let t1 = config.propose_timeout_for_round(1);
        let t2 = config.propose_timeout_for_round(2);

        assert!(t1 > t0);
        assert!(t2 > t1);
        assert_eq!(t1 - t0, config.timeout_delta);
    }
}
