//! Core consensus engine.
//!
//! The consensus engine coordinates agreement between validators
//! without violating the trust boundary constraints:
//!
//! - Never mutates state directly (MARS only)
//! - Never validates signatures directly (TEV only)
//! - Never handles network transport (POPEYE only)
//! - Never persists data directly (TAR only)
//!
//! Consensus decides WHICH block becomes canonical.

use crate::config::ConsensusConfig;
use crate::error::{ConsensusError, Result};
use crate::types::*;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Events emitted by the consensus engine.
#[derive(Debug, Clone)]
pub enum ConsensusEvent {
    /// Need to broadcast a proposal.
    BroadcastProposal(Proposal),
    /// Need to broadcast a prevote.
    BroadcastPrevote(Prevote),
    /// Need to broadcast a commit.
    BroadcastCommit(Commit),
    /// Block has been finalized.
    BlockFinalized {
        height: u64,
        block_hash: BlockHash,
        certificate: FinalityCertificate,
    },
    /// Round timed out, moving to next round.
    RoundTimeout { height: u64, round: u64 },
    /// Request to execute a block (calls MARS).
    ExecuteBlock {
        height: u64,
        prev_hash: BlockHash,
        transactions: Vec<u8>,
    },
}

/// Result of processing a consensus message.
#[derive(Debug)]
pub enum ProcessResult {
    /// Message processed, continue.
    Continue,
    /// Block finalized at this height.
    Finalized(FinalityCertificate),
    /// Need more votes.
    NeedMoreVotes,
    /// Message was stale/duplicate.
    Ignored,
}

/// The core consensus engine.
pub struct ConsensusEngine {
    /// Configuration.
    config: ConsensusConfig,
    /// Validator set.
    validator_set: ValidatorSet,
    /// Our validator keypair.
    signing_key: SigningKey,
    /// Our validator ID.
    our_id: ValidatorId,
    /// Current round state.
    state: RwLock<RoundState>,
    /// Finalized heights.
    finalized: RwLock<std::collections::HashMap<u64, FinalityCertificate>>,
    /// Event sender.
    event_tx: mpsc::UnboundedSender<ConsensusEvent>,
}

impl ConsensusEngine {
    /// Create a new consensus engine.
    pub fn new(
        config: ConsensusConfig,
        validator_set: ValidatorSet,
        signing_key: SigningKey,
        event_tx: mpsc::UnboundedSender<ConsensusEvent>,
    ) -> Self {
        let our_id = ValidatorId::from_verifying_key(&signing_key.verifying_key());

        Self {
            config,
            validator_set,
            signing_key,
            our_id,
            state: RwLock::new(RoundState::new(1, 0)),
            finalized: RwLock::new(std::collections::HashMap::new()),
            event_tx,
        }
    }

    /// Get our validator ID.
    pub fn our_id(&self) -> &ValidatorId {
        &self.our_id
    }

    /// Check if we are the leader for the current round.
    pub async fn is_leader(&self) -> bool {
        let state = self.state.read().await;
        let leader = self.validator_set.leader_for_round(state.round);
        leader.id == self.our_id
    }

    /// Get current height.
    pub async fn current_height(&self) -> u64 {
        self.state.read().await.height
    }

    /// Get current round.
    pub async fn current_round(&self) -> u64 {
        self.state.read().await.round
    }

    /// Start a new height (called after finalization or genesis).
    pub async fn start_height(&self, height: u64) -> Result<()> {
        let mut state = self.state.write().await;
        *state = RoundState::new(height, 0);

        info!(height, "Starting consensus for new height");

        // If we're the leader, we need to propose
        if self.validator_set.leader_for_round(0).id == self.our_id {
            info!(height, "We are the leader for round 0");
            // Emit event to request block execution from MARS
            let _ = self.event_tx.send(ConsensusEvent::ExecuteBlock {
                height,
                prev_hash: [0u8; 32], // Caller must provide actual prev_hash
                transactions: Vec::new(),
            });
        }

        Ok(())
    }

    /// Create and broadcast a proposal (called by leader after MARS execution).
    pub async fn propose(
        &self,
        prev_hash: BlockHash,
        block_hash: BlockHash,
        state_root: StateRoot,
        transactions: Vec<u8>,
    ) -> Result<()> {
        let state = self.state.read().await;

        // Verify we're the leader
        let leader = self.validator_set.leader_for_round(state.round);
        if leader.id != self.our_id {
            return Err(ConsensusError::WrongLeader {
                expected: leader.id.to_hex(),
                got: self.our_id.to_hex(),
            });
        }

        // Create proposal
        let mut proposal = Proposal {
            height: state.height,
            round: state.round,
            prev_hash,
            block_hash,
            state_root,
            transactions,
            proposer: self.our_id.clone(),
            signature: Signature64::default(),
        };

        // Sign it
        let payload = proposal.signing_payload();
        let signature = self.signing_key.sign(&payload);
        proposal.signature = Signature64::from_bytes(signature.to_bytes());

        info!(
            height = state.height,
            round = state.round,
            block_hash = hex::encode(&block_hash[..8]),
            "Broadcasting proposal"
        );

        // Broadcast
        let _ = self
            .event_tx
            .send(ConsensusEvent::BroadcastProposal(proposal));

        Ok(())
    }

    /// Process an incoming proposal.
    pub async fn on_proposal(&self, proposal: Proposal) -> Result<ProcessResult> {
        let mut state = self.state.write().await;

        // Check height and round
        if proposal.height != state.height {
            return Ok(ProcessResult::Ignored);
        }
        if proposal.round != state.round {
            return Ok(ProcessResult::Ignored);
        }

        // Verify it's from the correct leader
        let leader = self.validator_set.leader_for_round(state.round);
        if proposal.proposer != leader.id {
            warn!(
                expected = %leader.id,
                got = %proposal.proposer,
                "Proposal from wrong leader"
            );
            return Err(ConsensusError::WrongLeader {
                expected: leader.id.to_hex(),
                got: proposal.proposer.to_hex(),
            });
        }

        // Verify signature
        self.verify_proposal_signature(&proposal)?;

        // Store proposal
        state.proposal = Some(proposal.clone());
        state.phase = Phase::Prevote;

        info!(
            height = state.height,
            round = state.round,
            block_hash = hex::encode(&proposal.block_hash[..8]),
            "Received valid proposal, moving to prevote"
        );

        // If we haven't prevoted yet, vote for this block
        if !state.prevoted {
            drop(state); // Release lock before async operation
            self.prevote(Some(proposal.block_hash)).await?;
        }

        Ok(ProcessResult::Continue)
    }

    /// Cast a prevote.
    async fn prevote(&self, block_hash: Option<BlockHash>) -> Result<()> {
        let mut state = self.state.write().await;

        if state.prevoted {
            return Ok(()); // Already voted
        }

        let mut prevote = Prevote {
            height: state.height,
            round: state.round,
            block_hash,
            validator: self.our_id.clone(),
            signature: Signature64::default(),
        };

        let payload = prevote.signing_payload();
        let signature = self.signing_key.sign(&payload);
        prevote.signature = Signature64::from_bytes(signature.to_bytes());

        state.prevoted = true;

        debug!(
            height = state.height,
            round = state.round,
            block = block_hash.map(|h| hex::encode(&h[..8])),
            "Casting prevote"
        );

        let _ = self.event_tx.send(ConsensusEvent::BroadcastPrevote(prevote));

        Ok(())
    }

    /// Process an incoming prevote.
    pub async fn on_prevote(&self, prevote: Prevote) -> Result<ProcessResult> {
        let mut state = self.state.write().await;

        // Check height and round
        if prevote.height != state.height {
            return Ok(ProcessResult::Ignored);
        }
        if prevote.round != state.round {
            return Ok(ProcessResult::Ignored);
        }

        // Verify validator is known
        if !self.validator_set.contains(&prevote.validator) {
            return Err(ConsensusError::UnknownValidator {
                validator: prevote.validator.to_hex(),
            });
        }

        // Verify signature
        self.verify_prevote_signature(&prevote)?;

        // Add to prevote set
        if !state.prevotes.add(prevote.clone()) {
            return Ok(ProcessResult::Ignored); // Duplicate
        }

        debug!(
            height = state.height,
            round = state.round,
            from = %prevote.validator,
            votes = state.prevotes.count(),
            "Received prevote"
        );

        // Check for quorum
        if let Some(block_hash) = &state.proposal.as_ref().map(|p| p.block_hash) {
            let weight = state.prevotes.weight_for_block(block_hash, &self.validator_set);
            let quorum = self.validator_set.quorum_threshold();

            if weight >= quorum && !state.committed {
                info!(
                    height = state.height,
                    round = state.round,
                    weight,
                    quorum,
                    "Prevote quorum reached, moving to commit"
                );

                state.phase = Phase::Commit;
                state.locked_block = Some(*block_hash);
                state.locked_round = Some(state.round);

                // Cast commit vote
                drop(state);
                self.commit(*block_hash).await?;
            }
        }

        Ok(ProcessResult::Continue)
    }

    /// Cast a commit vote.
    async fn commit(&self, block_hash: BlockHash) -> Result<()> {
        let mut state = self.state.write().await;

        if state.committed {
            return Ok(()); // Already committed
        }

        let mut commit = Commit {
            height: state.height,
            round: state.round,
            block_hash,
            validator: self.our_id.clone(),
            signature: Signature64::default(),
        };

        let payload = commit.signing_payload();
        let signature = self.signing_key.sign(&payload);
        commit.signature = Signature64::from_bytes(signature.to_bytes());

        state.committed = true;

        info!(
            height = state.height,
            round = state.round,
            block_hash = hex::encode(&block_hash[..8]),
            "Casting commit vote"
        );

        let _ = self.event_tx.send(ConsensusEvent::BroadcastCommit(commit));

        Ok(())
    }

    /// Process an incoming commit.
    pub async fn on_commit(&self, commit: Commit) -> Result<ProcessResult> {
        let mut state = self.state.write().await;

        // Check height
        if commit.height != state.height {
            return Ok(ProcessResult::Ignored);
        }

        // Verify validator is known
        if !self.validator_set.contains(&commit.validator) {
            return Err(ConsensusError::UnknownValidator {
                validator: commit.validator.to_hex(),
            });
        }

        // Verify signature
        self.verify_commit_signature(&commit)?;

        // Add to commit set
        if !state.commits.add(commit.clone()) {
            return Ok(ProcessResult::Ignored); // Duplicate
        }

        debug!(
            height = state.height,
            from = %commit.validator,
            commits = state.commits.count(),
            "Received commit"
        );

        // Check for finality
        let weight = state
            .commits
            .weight_for_block(&commit.block_hash, &self.validator_set);
        let quorum = self.validator_set.quorum_threshold();

        if weight >= quorum {
            info!(
                height = state.height,
                round = state.round,
                block_hash = hex::encode(&commit.block_hash[..8]),
                weight,
                "BLOCK FINALIZED"
            );

            // Create finality certificate
            let commits = state.commits.commits_for_block(&commit.block_hash);
            let certificate = FinalityCertificate::new(
                state.height,
                commit.block_hash,
                commits,
                weight,
            );

            // Store finalized block
            let height = state.height;
            {
                let mut finalized = self.finalized.write().await;
                finalized.insert(height, certificate.clone());
            }

            // Emit finalization event
            let _ = self.event_tx.send(ConsensusEvent::BlockFinalized {
                height,
                block_hash: commit.block_hash,
                certificate: certificate.clone(),
            });

            // Advance to next height
            state.phase = Phase::Completed;

            return Ok(ProcessResult::Finalized(certificate));
        }

        Ok(ProcessResult::NeedMoreVotes)
    }

    /// Handle round timeout.
    pub async fn on_timeout(&self) -> Result<()> {
        let mut state = self.state.write().await;

        warn!(
            height = state.height,
            round = state.round,
            phase = %state.phase,
            "Round timeout"
        );

        // Emit timeout event
        let _ = self.event_tx.send(ConsensusEvent::RoundTimeout {
            height: state.height,
            round: state.round,
        });

        // Move to next round
        *state = state.next_round();

        info!(
            height = state.height,
            round = state.round,
            "Advanced to next round"
        );

        // If we're the new leader, request block execution
        if self.validator_set.leader_for_round(state.round).id == self.our_id {
            info!("We are the leader for round {}", state.round);
        }

        Ok(())
    }

    /// Verify proposal signature.
    fn verify_proposal_signature(&self, proposal: &Proposal) -> Result<()> {
        let validator = self
            .validator_set
            .get(&proposal.proposer)
            .ok_or_else(|| ConsensusError::UnknownValidator {
                validator: proposal.proposer.to_hex(),
            })?;

        let verifying_key = validator.verifying_key().ok_or_else(|| {
            ConsensusError::InvalidSignature {
                message_type: "proposal".to_string(),
            }
        })?;

        let signature = Signature::from_bytes(proposal.signature.as_bytes());
        let payload = proposal.signing_payload();

        verifying_key.verify(&payload, &signature).map_err(|_| {
            ConsensusError::InvalidSignature {
                message_type: "proposal".to_string(),
            }
        })
    }

    /// Verify prevote signature.
    fn verify_prevote_signature(&self, prevote: &Prevote) -> Result<()> {
        let validator = self
            .validator_set
            .get(&prevote.validator)
            .ok_or_else(|| ConsensusError::UnknownValidator {
                validator: prevote.validator.to_hex(),
            })?;

        let verifying_key = validator.verifying_key().ok_or_else(|| {
            ConsensusError::InvalidSignature {
                message_type: "prevote".to_string(),
            }
        })?;

        let signature = Signature::from_bytes(prevote.signature.as_bytes());
        let payload = prevote.signing_payload();

        verifying_key.verify(&payload, &signature).map_err(|_| {
            ConsensusError::InvalidSignature {
                message_type: "prevote".to_string(),
            }
        })
    }

    /// Verify commit signature.
    fn verify_commit_signature(&self, commit: &Commit) -> Result<()> {
        let validator = self
            .validator_set
            .get(&commit.validator)
            .ok_or_else(|| ConsensusError::UnknownValidator {
                validator: commit.validator.to_hex(),
            })?;

        let verifying_key = validator.verifying_key().ok_or_else(|| {
            ConsensusError::InvalidSignature {
                message_type: "commit".to_string(),
            }
        })?;

        let signature = Signature::from_bytes(commit.signature.as_bytes());
        let payload = commit.signing_payload();

        verifying_key.verify(&payload, &signature).map_err(|_| {
            ConsensusError::InvalidSignature {
                message_type: "commit".to_string(),
            }
        })
    }

    /// Check if a height has been finalized.
    pub async fn is_finalized(&self, height: u64) -> bool {
        self.finalized.read().await.contains_key(&height)
    }

    /// Get finality certificate for a height.
    pub async fn get_finality_certificate(&self, height: u64) -> Option<FinalityCertificate> {
        self.finalized.read().await.get(&height).cloned()
    }

    /// Fork choice: get the canonical block hash at a height.
    pub async fn fork_choice(&self, height: u64) -> Option<BlockHash> {
        // Rule 1: Prefer finalized block
        if let Some(cert) = self.finalized.read().await.get(&height) {
            return Some(cert.block_hash);
        }

        let state = self.state.read().await;

        // Only applies to current height
        if state.height != height {
            return None;
        }

        // Rule 2: Prefer block with highest commit quorum
        if let Some((block_hash, weight)) = state
            .commits
            .commits_for_block(&state.locked_block.unwrap_or([0u8; 32]))
            .iter()
            .map(|c| c.block_hash)
            .next()
            .map(|h| {
                (
                    h,
                    state.commits.weight_for_block(&h, &self.validator_set),
                )
            })
        {
            if weight > 0 {
                return Some(block_hash);
            }
        }

        // Rule 3: Fall back to locked block (if any)
        state.locked_block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    fn create_test_engine() -> (ConsensusEngine, mpsc::UnboundedReceiver<ConsensusEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let signing_key = SigningKey::generate(&mut OsRng);
        let pubkey = signing_key.verifying_key().to_bytes();

        let validator_set = ValidatorSet::new(vec![
            pubkey,
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        ]);

        let engine = ConsensusEngine::new(
            ConsensusConfig::default(),
            validator_set,
            signing_key,
            tx,
        );

        (engine, rx)
    }

    #[tokio::test]
    async fn engine_creation() {
        let (engine, _rx) = create_test_engine();
        assert_eq!(engine.current_height().await, 1);
        assert_eq!(engine.current_round().await, 0);
    }

    #[tokio::test]
    async fn start_new_height() {
        let (engine, _rx) = create_test_engine();
        engine.start_height(5).await.unwrap();
        assert_eq!(engine.current_height().await, 5);
        assert_eq!(engine.current_round().await, 0);
    }

    #[tokio::test]
    async fn timeout_advances_round() {
        let (engine, _rx) = create_test_engine();
        assert_eq!(engine.current_round().await, 0);

        engine.on_timeout().await.unwrap();

        assert_eq!(engine.current_round().await, 1);
        assert_eq!(engine.current_height().await, 1); // Same height
    }
}
