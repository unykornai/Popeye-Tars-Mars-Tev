//! Core consensus types.
//!
//! Data structures for the BFT consensus protocol:
//! - Validator identity and sets
//! - Round state machine
//! - Proposals, prevotes, and commits
//! - Finality certificates

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A 32-byte block hash.
pub type BlockHash = [u8; 32];

/// A 32-byte state root.
pub type StateRoot = [u8; 32];

/// A 64-byte signature with serde support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature64(pub [u8; 64]);

impl Default for Signature64 {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl Signature64 {
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl Serialize for Signature64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Signature64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("signature must be 64 bytes"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(Signature64(arr))
    }
}

/// Validator identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidatorId(pub [u8; 32]);

impl ValidatorId {
    /// Create validator ID from public key bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create from verifying key.
    pub fn from_verifying_key(key: &VerifyingKey) -> Self {
        Self(key.to_bytes())
    }

    /// Get the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hex representation for logging.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0[..8]) // Short form
    }
}

impl std::fmt::Display for ValidatorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// A validator with its public key and weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Unique identifier (derived from public key).
    pub id: ValidatorId,
    /// Ed25519 public key bytes.
    pub pubkey: [u8; 32],
    /// Voting weight (1 for now, extensible for staking).
    pub weight: u64,
}

impl Validator {
    /// Create a new validator with weight 1.
    pub fn new(pubkey: [u8; 32]) -> Self {
        Self {
            id: ValidatorId::from_bytes(pubkey),
            pubkey,
            weight: 1,
        }
    }

    /// Get the verifying key.
    pub fn verifying_key(&self) -> Option<VerifyingKey> {
        VerifyingKey::from_bytes(&self.pubkey).ok()
    }
}

/// The set of active validators for a given epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// Ordered list of validators (order matters for leader selection).
    validators: Vec<Validator>,
    /// Quick lookup by ID.
    #[serde(skip)]
    by_id: HashMap<ValidatorId, usize>,
    /// Total voting weight.
    total_weight: u64,
}

impl ValidatorSet {
    /// Create a new validator set from a list of public keys.
    pub fn new(pubkeys: Vec<[u8; 32]>) -> Self {
        let validators: Vec<Validator> = pubkeys.into_iter().map(Validator::new).collect();
        let total_weight = validators.iter().map(|v| v.weight).sum();
        let by_id = validators
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id.clone(), i))
            .collect();

        Self {
            validators,
            by_id,
            total_weight,
        }
    }

    /// Rebuild the lookup index after deserialization.
    pub fn rebuild_index(&mut self) {
        self.by_id = self
            .validators
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id.clone(), i))
            .collect();
    }

    /// Number of validators.
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Get validator by ID.
    pub fn get(&self, id: &ValidatorId) -> Option<&Validator> {
        self.by_id.get(id).map(|&i| &self.validators[i])
    }

    /// Check if a validator ID is in the set.
    pub fn contains(&self, id: &ValidatorId) -> bool {
        self.by_id.contains_key(id)
    }

    /// Get the leader for a given round (deterministic rotation).
    pub fn leader_for_round(&self, round: u64) -> &Validator {
        let index = (round as usize) % self.validators.len();
        &self.validators[index]
    }

    /// Calculate quorum threshold (2/3 + 1 of total weight).
    pub fn quorum_threshold(&self) -> u64 {
        // For BFT: need > 2/3, so we use 2*total/3 + 1
        (self.total_weight * 2) / 3 + 1
    }

    /// Calculate the maximum faulty validators tolerated.
    pub fn max_faulty(&self) -> u64 {
        // f < n/3, so max f = (n-1)/3
        (self.total_weight - 1) / 3
    }

    /// Get total voting weight.
    pub fn total_weight(&self) -> u64 {
        self.total_weight
    }

    /// Iterate over validators.
    pub fn iter(&self) -> impl Iterator<Item = &Validator> {
        self.validators.iter()
    }
}

/// Consensus round phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Waiting for proposal from leader.
    Propose,
    /// Voting on proposal validity.
    Prevote,
    /// Committing to finalize.
    Commit,
    /// Round completed (success or timeout).
    Completed,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Propose => write!(f, "Propose"),
            Phase::Prevote => write!(f, "Prevote"),
            Phase::Commit => write!(f, "Commit"),
            Phase::Completed => write!(f, "Completed"),
        }
    }
}

/// A block proposal from the round leader.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Block height.
    pub height: u64,
    /// Consensus round number.
    pub round: u64,
    /// Hash of the previous block.
    pub prev_hash: BlockHash,
    /// Proposed block hash.
    pub block_hash: BlockHash,
    /// State root after executing transactions.
    pub state_root: StateRoot,
    /// Serialized transactions (opaque to consensus).
    pub transactions: Vec<u8>,
    /// Proposer's validator ID.
    pub proposer: ValidatorId,
    /// Signature over the proposal.
    pub signature: Signature64,
}

impl Proposal {
    /// Create the signing payload for a proposal.
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"PROPOSAL");
        payload.extend_from_slice(&self.height.to_le_bytes());
        payload.extend_from_slice(&self.round.to_le_bytes());
        payload.extend_from_slice(&self.prev_hash);
        payload.extend_from_slice(&self.block_hash);
        payload.extend_from_slice(&self.state_root);
        payload
    }
}

/// A prevote for or against a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prevote {
    /// Block height.
    pub height: u64,
    /// Consensus round.
    pub round: u64,
    /// Block hash being voted for (None = nil vote).
    pub block_hash: Option<BlockHash>,
    /// Voter's validator ID.
    pub validator: ValidatorId,
    /// Signature over the vote.
    pub signature: Signature64,
}

impl Prevote {
    /// Create the signing payload for a prevote.
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"PREVOTE");
        payload.extend_from_slice(&self.height.to_le_bytes());
        payload.extend_from_slice(&self.round.to_le_bytes());
        match &self.block_hash {
            Some(hash) => payload.extend_from_slice(hash),
            None => payload.extend_from_slice(&[0u8; 32]),
        }
        payload
    }

    /// Check if this is a nil vote.
    pub fn is_nil(&self) -> bool {
        self.block_hash.is_none()
    }
}

/// A commit vote for finalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Block height.
    pub height: u64,
    /// Consensus round.
    pub round: u64,
    /// Block hash being committed.
    pub block_hash: BlockHash,
    /// Committer's validator ID.
    pub validator: ValidatorId,
    /// Signature over the commit.
    pub signature: Signature64,
}

impl Commit {
    /// Create the signing payload for a commit.
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"COMMIT");
        payload.extend_from_slice(&self.height.to_le_bytes());
        payload.extend_from_slice(&self.round.to_le_bytes());
        payload.extend_from_slice(&self.block_hash);
        payload
    }
}

/// Aggregated commit signatures proving finality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityCertificate {
    /// Block height.
    pub height: u64,
    /// Finalized block hash.
    pub block_hash: BlockHash,
    /// Commits from validators (must have quorum weight).
    pub commits: Vec<Commit>,
    /// Total weight of commits.
    pub total_weight: u64,
}

impl FinalityCertificate {
    /// Create a new finality certificate.
    pub fn new(height: u64, block_hash: BlockHash, commits: Vec<Commit>, total_weight: u64) -> Self {
        Self {
            height,
            block_hash,
            commits,
            total_weight,
        }
    }
}

/// Collection of prevotes for a round.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrevoteSet {
    /// Prevotes indexed by validator.
    votes: HashMap<ValidatorId, Prevote>,
    /// Votes grouped by block hash.
    by_block: HashMap<BlockHash, HashSet<ValidatorId>>,
    /// Nil votes.
    nil_votes: HashSet<ValidatorId>,
}

impl PrevoteSet {
    /// Create empty prevote set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a prevote, returns true if new.
    pub fn add(&mut self, prevote: Prevote) -> bool {
        let validator = prevote.validator.clone();

        if self.votes.contains_key(&validator) {
            return false; // Duplicate
        }

        match &prevote.block_hash {
            Some(hash) => {
                self.by_block
                    .entry(*hash)
                    .or_default()
                    .insert(validator.clone());
            }
            None => {
                self.nil_votes.insert(validator.clone());
            }
        }

        self.votes.insert(validator, prevote);
        true
    }

    /// Check if we have a vote from this validator.
    pub fn has_vote(&self, validator: &ValidatorId) -> bool {
        self.votes.contains_key(validator)
    }

    /// Get total weight voting for a specific block.
    pub fn weight_for_block(&self, block_hash: &BlockHash, validator_set: &ValidatorSet) -> u64 {
        self.by_block
            .get(block_hash)
            .map(|voters| {
                voters
                    .iter()
                    .filter_map(|v| validator_set.get(v))
                    .map(|v| v.weight)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Get the block hash with the most votes (if any).
    pub fn leading_block(&self, validator_set: &ValidatorSet) -> Option<(BlockHash, u64)> {
        self.by_block
            .iter()
            .map(|(hash, _)| (*hash, self.weight_for_block(hash, validator_set)))
            .max_by_key(|(_, weight)| *weight)
    }

    /// Total votes collected.
    pub fn count(&self) -> usize {
        self.votes.len()
    }
}

/// Collection of commits for a round.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommitSet {
    /// Commits indexed by validator.
    commits: HashMap<ValidatorId, Commit>,
    /// Commits grouped by block hash.
    by_block: HashMap<BlockHash, Vec<Commit>>,
}

impl CommitSet {
    /// Create empty commit set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a commit, returns true if new.
    pub fn add(&mut self, commit: Commit) -> bool {
        let validator = commit.validator.clone();

        if self.commits.contains_key(&validator) {
            return false; // Duplicate
        }

        let block_hash = commit.block_hash;
        self.by_block
            .entry(block_hash)
            .or_default()
            .push(commit.clone());

        self.commits.insert(validator, commit);
        true
    }

    /// Check if we have a commit from this validator.
    pub fn has_commit(&self, validator: &ValidatorId) -> bool {
        self.commits.contains_key(validator)
    }

    /// Get total weight committing to a specific block.
    pub fn weight_for_block(&self, block_hash: &BlockHash, validator_set: &ValidatorSet) -> u64 {
        self.by_block
            .get(block_hash)
            .map(|commits| {
                commits
                    .iter()
                    .filter_map(|c| validator_set.get(&c.validator))
                    .map(|v| v.weight)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Get commits for a block hash.
    pub fn commits_for_block(&self, block_hash: &BlockHash) -> Vec<Commit> {
        self.by_block.get(block_hash).cloned().unwrap_or_default()
    }

    /// Total commits collected.
    pub fn count(&self) -> usize {
        self.commits.len()
    }
}

/// Current state of a consensus round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundState {
    /// Block height being decided.
    pub height: u64,
    /// Current round number.
    pub round: u64,
    /// Current phase.
    pub phase: Phase,
    /// Proposal for this round (if received).
    pub proposal: Option<Proposal>,
    /// Collected prevotes.
    pub prevotes: PrevoteSet,
    /// Collected commits.
    pub commits: CommitSet,
    /// Whether we have prevoted.
    pub prevoted: bool,
    /// Whether we have committed.
    pub committed: bool,
    /// Block hash we locked on (if any).
    pub locked_block: Option<BlockHash>,
    /// Round we locked on.
    pub locked_round: Option<u64>,
}

impl RoundState {
    /// Create initial state for a height/round.
    pub fn new(height: u64, round: u64) -> Self {
        Self {
            height,
            round,
            phase: Phase::Propose,
            proposal: None,
            prevotes: PrevoteSet::new(),
            commits: CommitSet::new(),
            prevoted: false,
            committed: false,
            locked_block: None,
            locked_round: None,
        }
    }

    /// Advance to next round (same height).
    pub fn next_round(&self) -> Self {
        Self {
            height: self.height,
            round: self.round + 1,
            phase: Phase::Propose,
            proposal: None,
            prevotes: PrevoteSet::new(),
            commits: CommitSet::new(),
            prevoted: false,
            committed: false,
            locked_block: self.locked_block, // Carry forward lock
            locked_round: self.locked_round,
        }
    }

    /// Advance to next height.
    pub fn next_height(&self) -> Self {
        Self::new(self.height + 1, 0)
    }
}

/// Consensus message wrapper for network transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Block proposal.
    Proposal(Proposal),
    /// Prevote.
    Prevote(Prevote),
    /// Commit.
    Commit(Commit),
}

impl ConsensusMessage {
    /// Get the height this message is for.
    pub fn height(&self) -> u64 {
        match self {
            ConsensusMessage::Proposal(p) => p.height,
            ConsensusMessage::Prevote(p) => p.height,
            ConsensusMessage::Commit(c) => c.height,
        }
    }

    /// Get the round this message is for.
    pub fn round(&self) -> u64 {
        match self {
            ConsensusMessage::Proposal(p) => p.round,
            ConsensusMessage::Prevote(p) => p.round,
            ConsensusMessage::Commit(c) => c.round,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_validator_set() -> ValidatorSet {
        let keys: Vec<[u8; 32]> = (0..4).map(|i| [i as u8; 32]).collect();
        ValidatorSet::new(keys)
    }

    #[test]
    fn validator_set_quorum() {
        let vs = test_validator_set();
        assert_eq!(vs.len(), 4);
        assert_eq!(vs.total_weight(), 4);
        // Quorum = 2*4/3 + 1 = 3
        assert_eq!(vs.quorum_threshold(), 3);
        // Max faulty = (4-1)/3 = 1
        assert_eq!(vs.max_faulty(), 1);
    }

    #[test]
    fn leader_rotation() {
        let vs = test_validator_set();
        let l0 = vs.leader_for_round(0);
        let l1 = vs.leader_for_round(1);
        let l4 = vs.leader_for_round(4);

        assert_ne!(l0.id, l1.id);
        assert_eq!(l0.id, l4.id); // Wraps around
    }

    #[test]
    fn prevote_set_aggregation() {
        let vs = test_validator_set();
        let mut prevotes = PrevoteSet::new();

        let block_hash = [1u8; 32];

        // Add prevotes from 3 validators for the same block
        for i in 0..3 {
            let prevote = Prevote {
                height: 1,
                round: 0,
                block_hash: Some(block_hash),
                validator: ValidatorId([i as u8; 32]),
                signature: Signature64::default(),
            };
            assert!(prevotes.add(prevote));
        }

        assert_eq!(prevotes.count(), 3);
        assert_eq!(prevotes.weight_for_block(&block_hash, &vs), 3);
    }

    #[test]
    fn duplicate_vote_rejected() {
        let mut prevotes = PrevoteSet::new();

        let prevote = Prevote {
            height: 1,
            round: 0,
            block_hash: Some([1u8; 32]),
            validator: ValidatorId([0u8; 32]),
            signature: Signature64::default(),
        };

        assert!(prevotes.add(prevote.clone()));
        assert!(!prevotes.add(prevote)); // Duplicate
    }

    #[test]
    fn round_state_progression() {
        let mut state = RoundState::new(1, 0);
        assert_eq!(state.phase, Phase::Propose);

        let next_round = state.next_round();
        assert_eq!(next_round.round, 1);
        assert_eq!(next_round.height, 1);

        let next_height = state.next_height();
        assert_eq!(next_height.height, 2);
        assert_eq!(next_height.round, 0);
    }
}
