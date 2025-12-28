//! DAO Governor Contract
//!
//! A decentralized autonomous organization governance contract for proposal voting and execution.
//!
//! ## Features
//! - Proposal Creation and Management - Submit and track governance proposals
//! - Token-Based Voting - Vote with governance token weight
//! - Quorum Enforcement - Require minimum participation for valid proposals
//! - Voting Periods - Configurable voting windows
//! - Delegation Support - Delegate voting power to other addresses
//! - Timelock Integration - Queue successful proposals for delayed execution
//! - Multiple Choice Voting - Support for/against/abstain choices
//! - Proposal States - Track complete proposal lifecycle

#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use silica_contract_sdk::event;
use silica_contract_sdk::prelude::*;
// use silica_contract_sdk::storage::Vector; // Unused import
use serde::{Deserialize, Serialize};

/// Vote choice enumeration
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoteType {
    AGAINST = 0,
    FOR = 1,
    ABSTAIN = 2,
}

/// Proposal state enumeration
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    PENDING = 0,   // Created, waiting for voting delay
    ACTIVE = 1,    // Voting is open
    SUCCEEDED = 2, // Vote passed, ready for execution
    DEFEATED = 3,  // Vote failed
    EXECUTED = 4,  // Proposal executed successfully
    CANCELLED = 5, // Proposal cancelled
    EXPIRED = 6,   // Voting period expired without execution
}

/// Vote structure
#[derive(Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub choice: VoteType,
    pub weight: u64,
    pub timestamp: u64,
}

/// Proposal structure
#[derive(Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub targets: Vec<String>,
    pub values: Vec<u64>,
    pub calldatas: Vec<Vec<u8>>,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub executed: bool,
    pub cancelled: bool,
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub executed_timestamp: u64,
    pub timelock_id: Option<[u8; 32]>,
}

/// Governor configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct GovernorConfig {
    pub name: String,
    pub governance_token: String,
    pub timelock_contract: String,
    pub voting_delay: u64,
    pub voting_period: u64,
    pub proposal_threshold: u64,
    pub quorum_votes: u64,
    pub proposal_count: u64,
    pub initialized: bool,
}

/// Delegate information
#[derive(Serialize, Deserialize)]
pub struct DelegateInfo {
    pub delegate: String,
    pub votes: u64,
    pub timestamp: u64,
}

/// Initialize the DAO governor contract
///
/// # Arguments
/// * `name` - Governor contract name
/// * `governance_token` - Address of governance token contract
/// * `timelock_contract` - Address of timelock contract for execution
/// * `voting_delay` - Blocks to wait before voting starts
/// * `voting_period` - Duration of voting period in blocks
/// * `proposal_threshold` - Minimum token weight to create proposals
/// * `quorum_votes` - Minimum votes required for valid proposal
#[unsafe(no_mangle)]
pub extern "C" fn initialize(
    name: String,
    governance_token: String,
    timelock_contract: String,
    voting_delay: u64,
    voting_period: u64,
    proposal_threshold: u64,
    quorum_votes: u64,
) {
    let ctx = context();
    let deployer = ctx.sender();

    // Validate parameters
    if name.is_empty() {
        log("Governor name is required");
        return;
    }

    if governance_token.is_empty() {
        log("Governance token address is required");
        return;
    }

    if timelock_contract.is_empty() {
        log("Timelock contract address is required");
        return;
    }

    if voting_delay == 0 {
        log("Voting delay must be > 0");
        return;
    }

    if voting_period == 0 {
        log("Voting period must be > 0");
        return;
    }

    if proposal_threshold == 0 {
        log("Proposal threshold must be > 0");
        return;
    }

    if quorum_votes == 0 {
        log("Quorum votes must be > 0");
        return;
    }

    // Initialize governor configuration
    let config = GovernorConfig {
        name: name.clone(),
        governance_token: governance_token.clone(),
        timelock_contract: timelock_contract.clone(),
        voting_delay,
        voting_period,
        proposal_threshold,
        quorum_votes,
        proposal_count: 0,
        initialized: true,
    };

    // Initialize access control with deployer as owner
    AccessControl::initialize(&deployer);

    let mut storage_ref = storage();
    if storage_ref.set("config", &config).is_err() {
        log("Failed to store governor config");
        return;
    }

    log(&format!(
        "DAO Governor '{}' initialized with token: {}, timelock: {}, delay: {}, period: {}",
        name, governance_token, timelock_contract, voting_delay, voting_period
    ));
    event!("GovernorInitialized",
        name: name,
        governance_token: governance_token,
        timelock_contract: timelock_contract,
        voting_delay: voting_delay,
        voting_period: voting_period,
        proposal_threshold: proposal_threshold,
        quorum_votes: quorum_votes
    );
}

/// Get current timestamp
fn get_timestamp() -> u64 {
    let ctx = context();
    ctx.block_timestamp()
}

/// Check if caller has sufficient voting power to create proposals
fn has_proposal_power(caller: &str) -> bool {
    let storage_ref = storage();
    let config: GovernorConfig = match storage_ref.get::<GovernorConfig>("config") {
        Ok(Some(c)) => c,
        _ => return false,
    };

    let balances: Map<String, u64> = Map::new("balances");
    let balance = match balances.get(&caller.to_string()) {
        Ok(Some(b)) => b,
        _ => return false,
    };

    balance >= config.proposal_threshold
}

/// Create a new governance proposal
///
/// # Arguments
/// * `title` - Short proposal title
/// * `description` - Detailed proposal description
/// * `targets` - Array of contract addresses to call
/// * `values` - Array of CHERT amounts to send
/// * `calldatas` - Array of call data for each target
///
/// # Returns
/// Proposal ID
#[unsafe(no_mangle)]
pub extern "C" fn propose(
    title: String,
    description: String,
    targets: Vec<String>,
    values: Vec<u64>,
    calldatas: Vec<Vec<u8>>,
) -> u64 {
    let ctx = context();
    let proposer = ctx.sender();

    // Check if caller has proposal power
    if !has_proposal_power(&proposer) {
        log("Insufficient voting power to create proposals");
        return 0;
    }

    // Validate parameters
    if title.is_empty() {
        log("Proposal title is required");
        return 0;
    }

    if description.is_empty() {
        log("Proposal description is required");
        return 0;
    }

    if targets.is_empty() {
        log("At least one target contract required");
        return 0;
    }

    if targets.len() != values.len() || targets.len() != calldatas.len() {
        log("Target, value, and calldata arrays must have same length");
        return 0;
    }

    // Get governor configuration
    let storage_ref = storage();
    let config: GovernorConfig = match storage_ref.get::<GovernorConfig>("config") {
        Ok(Some(c)) => c,
        _ => {
            log("Failed to load governor config");
            return 0;
        }
    };

    // Create proposal
    let current_time = get_timestamp();
    let start_time = current_time + config.voting_delay;
    let end_time = start_time + config.voting_period;

    let proposal = Proposal {
        id: config.proposal_count,
        proposer: proposer.to_string(),
        title: title.clone(),
        description: description.clone(),
        targets: targets.clone(),
        values: values.clone(),
        calldatas: calldatas.clone(),
        start_timestamp: start_time,
        end_timestamp: end_time,
        executed: false,
        cancelled: false,
        for_votes: 0,
        against_votes: 0,
        abstain_votes: 0,
        executed_timestamp: 0,
        timelock_id: None,
    };

    // Store proposal
    let mut proposals: Map<u64, Proposal> = Map::new("proposals");
    if proposals.set(&config.proposal_count, &proposal).is_err() {
        log("Failed to store proposal");
        return 0;
    }

    // Track proposal in user's proposals
    let mut user_proposals: Map<String, Vec<u64>> = Map::new("user_proposals");
    let mut user_proposal_ids = match user_proposals.get(&proposer.to_string()) {
        Ok(Some(ids)) => ids,
        Ok(None) => Vec::new(),
        Err(_) => Vec::new(),
    };
    user_proposal_ids.push(config.proposal_count);
    if user_proposals
        .set(&proposer.to_string(), &user_proposal_ids)
        .is_err()
    {
        log("Failed to track user's proposals");
        return 0;
    }

    // Update proposal count
    let config_clone = config.clone();
    let mut config_mut = config_clone;
    config_mut.proposal_count += 1;
    let mut storage_ref = storage();
    if storage_ref.set("config", &config_mut).is_err() {
        log("Failed to update proposal count");
        return 0;
    }

    log(&format!(
        "Proposal {} created by {}: {}",
        config.proposal_count, proposer, title
    ));
    event!("ProposalCreated",
        proposal_id: config.proposal_count,
        proposer: proposer,
        title: title,
        start_time: start_time,
        end_time: end_time
    );

    config.proposal_count
}

/// Cast a vote on a proposal
///
/// # Arguments
/// * `proposal_id` - ID of the proposal to vote on
/// * `choice` - Vote choice (0=against, 1=for, 2=abstain)
///
/// # Returns
/// True if vote was cast successfully
#[unsafe(no_mangle)]
pub extern "C" fn cast_vote(proposal_id: u64, choice: u8) -> bool {
    let ctx = context();
    let voter = ctx.sender();

    // Check if vote choice is valid
    if choice > 2 {
        log("Invalid vote choice");
        return false;
    }

    // Check if proposal exists
    let storage_ref = storage();
    let mut proposals: Map<u64, Proposal> = Map::new("proposals");
    let mut proposal = match proposals.get(&proposal_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            log("Proposal does not exist");
            return false;
        }
        Err(_) => {
            log("Failed to read proposal");
            return false;
        }
    };

    // Check if voting period is active
    let current_time = get_timestamp();
    if current_time < proposal.start_timestamp {
        log("Voting has not started yet");
        return false;
    }

    if current_time > proposal.end_timestamp {
        log("Voting period has ended");
        return false;
    }

    if proposal.executed || proposal.cancelled {
        log("Cannot vote on executed or cancelled proposal");
        return false;
    }

    // Check if voter has already voted
    let mut votes: Map<(u64, String), Vote> = Map::new("votes");
    if votes
        .get(&(proposal_id, voter.to_string()))
        .ok()
        .flatten()
        .is_some()
    {
        log("Already voted on this proposal");
        return false;
    }

    // Get voter's voting power (token balance)
    let balances: Map<String, u64> = Map::new("balances");
    let voting_power = match balances.get(&voter.to_string()) {
        Ok(Some(balance)) => balance,
        _ => {
            log("No voting power found");
            return false;
        }
    };

    // Cast vote
    let vote_choice = match choice {
        0 => VoteType::AGAINST,
        1 => VoteType::FOR,
        2 => VoteType::ABSTAIN,
        _ => VoteType::FOR, // Fallback, shouldn't reach here
    };

    let vote = Vote {
        voter: voter.to_string(),
        choice: vote_choice,
        weight: voting_power,
        timestamp: current_time,
    };

    // Store vote
    if votes.set(&(proposal_id, voter.to_string()), &vote).is_err() {
        log("Failed to store vote");
        return false;
    }

    // Update proposal vote counts
    match vote_choice {
        VoteType::FOR => proposal.for_votes += voting_power,
        VoteType::AGAINST => proposal.against_votes += voting_power,
        VoteType::ABSTAIN => proposal.abstain_votes += voting_power,
    }

    // Update proposal in storage
    if proposals.set(&proposal_id, &proposal).is_err() {
        log("Failed to update proposal");
        return false;
    }

    log(&format!(
        "Vote cast on proposal {} by {}: {:?} with weight {}",
        proposal_id, voter, vote_choice, voting_power
    ));
    event!("VoteCast",
        proposal_id: proposal_id,
        voter: voter,
        choice: choice,
        weight: voting_power
    );

    true
}

/// Delegate voting power to another address
///
/// # Arguments
/// * `delegatee` - Address to delegate voting power to
#[unsafe(no_mangle)]
pub extern "C" fn delegate(delegatee: String) {
    let ctx = context();
    let delegator = ctx.sender();

    if delegatee.is_empty() {
        log("Delegatee address is required");
        return;
    }

    if delegatee == delegator {
        log("Cannot delegate to yourself");
        return;
    }

    // Get delegator's current voting power
    let balances: Map<String, u64> = Map::new("balances");
    let voting_power = match balances.get(&delegator.to_string()) {
        Ok(Some(balance)) => balance,
        _ => {
            log("No voting power found");
            return;
        }
    };

    // Store delegation
    let mut delegations: Map<String, String> = Map::new("delegations");
    if delegations.set(&delegator.to_string(), &delegatee).is_err() {
        log("Failed to store delegation");
        return;
    }

    // Update delegatee's total delegated votes
    let mut delegate_votes: Map<String, u64> = Map::new("delegate_votes");
    let current_delegate_votes = match delegate_votes.get(&delegatee) {
        Ok(Some(votes)) => votes,
        _ => 0,
    };

    if delegate_votes
        .set(&delegatee, &(current_delegate_votes + voting_power))
        .is_err()
    {
        log("Failed to update delegate votes");
        return;
    }

    log(&format!(
        "Delegated {} votes from {} to {}",
        voting_power, delegator, delegatee
    ));
    event!("DelegateChanged",
        delegator: delegator,
        delegatee: delegatee,
        new_weight: current_delegate_votes + voting_power
    );
}

/// Execute a successful proposal through the timelock
///
/// # Arguments
/// * `proposal_id` - ID of the proposal to execute
#[unsafe(no_mangle)]
pub extern "C" fn execute(proposal_id: u64) {
    let ctx = context();
    let executor = ctx.sender();

    // Check if proposal exists and can be executed
    let storage_ref = storage();
    let mut proposals: Map<u64, Proposal> = Map::new("proposals");
    let mut proposal = match proposals.get(&proposal_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            log("Proposal does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read proposal");
            return;
        }
    };

    // Check if voting period has ended
    let current_time = get_timestamp();
    if current_time <= proposal.end_timestamp {
        log("Voting period has not ended yet");
        return;
    }

    if proposal.executed {
        log("Proposal already executed");
        return;
    }

    if proposal.cancelled {
        log("Cannot execute cancelled proposal");
        return;
    }

    // Check if proposal succeeded
    let config: GovernorConfig = match storage_ref.get("config") {
        Ok(Some(c)) => c,
        _ => {
            log("Failed to load governor config");
            return;
        }
    };

    if proposal.for_votes <= proposal.against_votes {
        log("Proposal did not pass");
        proposal.cancelled = true;
        if proposals.set(&proposal_id, &proposal).is_err() {
            log("Failed to update proposal");
            return;
        }
        return;
    }

    if proposal.for_votes < config.quorum_votes {
        log("Proposal did not reach quorum");
        proposal.cancelled = true;
        if proposals.set(&proposal_id, &proposal).is_err() {
            log("Failed to update proposal");
            return;
        }
        return;
    }

    // Queue proposal in timelock
    if proposal.targets.len() > 1 {
        // Schedule batch operation
        // In real implementation, this would call timelock.schedule_batch()
        log(&format!(
            "Queueing batch proposal {} in timelock with {} operations",
            proposal_id,
            proposal.targets.len()
        ));

        // For now, simulate a timelock ID
        let timelock_id = [proposal_id as u8; 32];
        proposal.timelock_id = Some(timelock_id);
    } else {
        // Schedule single operation
        // In real implementation, this would call timelock.schedule()
        log(&format!(
            "Queueing proposal {} in timelock: {} -> {}",
            proposal_id, proposal.targets[0], proposal.title
        ));

        // For now, simulate a timelock ID
        let timelock_id = [proposal_id as u8; 32];
        proposal.timelock_id = Some(timelock_id);
    }

    // Mark proposal as executed
    proposal.executed = true;
    proposal.executed_timestamp = current_time;

    if proposals.set(&proposal_id, &proposal).is_err() {
        log("Failed to update proposal");
        return;
    }

    log(&format!(
        "Proposal {} executed successfully by {}",
        proposal_id, executor
    ));
    event!("ProposalExecuted",
        proposal_id: proposal_id,
        executor: executor,
        timelock_id: "set" // Placeholder - would need real hash formatting
    );
}

/// Cancel a proposal
///
/// # Arguments
/// * `proposal_id` - ID of the proposal to cancel
#[unsafe(no_mangle)]
pub extern "C" fn cancel(proposal_id: u64) {
    let ctx = context();
    let canceller = ctx.sender();

    // Check if proposal exists
    let mut proposals: Map<u64, Proposal> = Map::new("proposals");
    let mut proposal = match proposals.get(&proposal_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            log("Proposal does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read proposal");
            return;
        }
    };

    if proposal.executed {
        log("Cannot cancel executed proposal");
        return;
    }

    if proposal.cancelled {
        log("Proposal is already cancelled");
        return;
    }

    // Only proposer can cancel (or anyone after expiry)
    let current_time = get_timestamp();
    if proposal.proposer != canceller && current_time <= proposal.end_timestamp {
        log("Only proposer can cancel active proposals");
        return;
    }

    // Cancel proposal
    proposal.cancelled = true;
    if proposals.set(&proposal_id, &proposal).is_err() {
        log("Failed to update proposal");
        return;
    }

    log(&format!(
        "Proposal {} cancelled by {}",
        proposal_id, canceller
    ));
    event!("ProposalCancelled", proposal_id: proposal_id, canceller: canceller);
}

/// Get the current state of a proposal
#[unsafe(no_mangle)]
pub extern "C" fn state(proposal_id: u64) -> u8 {
    let storage_ref = storage();
    let proposals: Map<u64, Proposal> = Map::new("proposals");

    match proposals.get(&proposal_id) {
        Ok(Some(proposal)) => {
            let current_time = get_timestamp();

            if proposal.executed {
                return ProposalState::EXECUTED as u8;
            }

            if proposal.cancelled {
                return ProposalState::CANCELLED as u8;
            }

            if current_time < proposal.start_timestamp {
                return ProposalState::PENDING as u8;
            }

            if current_time >= proposal.start_timestamp && current_time <= proposal.end_timestamp {
                return ProposalState::ACTIVE as u8;
            }

            // Voting ended, determine result
            let config: GovernorConfig = match storage_ref.get("config") {
                Ok(Some(c)) => c,
                _ => return ProposalState::DEFEATED as u8,
            };

            if proposal.for_votes > proposal.against_votes
                && proposal.for_votes >= config.quorum_votes
            {
                ProposalState::SUCCEEDED as u8
            } else {
                ProposalState::DEFEATED as u8
            }
        }
        Ok(None) => {
            log("Proposal does not exist");
            ProposalState::DEFEATED as u8
        }
        Err(_) => {
            log("Failed to read proposal");
            ProposalState::DEFEATED as u8
        }
    }
}

/// Query function: Get the number of proposals created
#[unsafe(no_mangle)]
pub extern "C" fn proposal_count() {
    let storage_ref = storage();
    match storage_ref.get::<GovernorConfig>("config") {
        Ok(Some(c)) => c.proposal_count,
        _ => 0,
    };
}

/// Query function: Get proposal details
#[unsafe(no_mangle)]
pub extern "C" fn get_proposal(proposal_id: u64) -> u64 {
    let storage_ref = storage();
    let proposals: Map<u64, Proposal> = Map::new("proposals");

    match proposals.get(&proposal_id) {
        Ok(Some(_proposal)) => {
            log(&format!("Proposal {} exists", proposal_id));
            1 // Return 1 to indicate existence
        }
        _ => {
            log(&format!("Proposal {} not found", proposal_id));
            0 // Return 0 to indicate not found
        }
    }
}

/// Query function: Check if an account has voted on a proposal
#[unsafe(no_mangle)]
pub extern "C" fn has_voted(proposal_id: u64, voter: String) -> bool {
    let votes: Map<(u64, String), Vote> = Map::new("votes");
    match votes.get(&(proposal_id, voter)) {
        Ok(Some(_vote)) => true,
        _ => false,
    }
}

/// Query function: Get the current voting power of an account
#[unsafe(no_mangle)]
pub extern "C" fn get_voting_power(account: String) -> u64 {
    // Get direct balance
    let balances: Map<String, u64> = Map::new("balances");
    let mut total_power = match balances.get(&account) {
        Ok(Some(balance)) => balance,
        _ => 0,
    };

    // Add delegated power
    let delegate_votes: Map<String, u64> = Map::new("delegate_votes");
    if let Ok(Some(delegated_votes)) = delegate_votes.get(&account) {
        total_power += delegated_votes;
    }

    total_power
}

/// Query function: Get the current governor configuration
#[unsafe(no_mangle)]
pub extern "C" fn get_config() -> u64 {
    let storage_ref = storage();
    let config: GovernorConfig = match storage_ref.get::<GovernorConfig>("config") {
        Ok(Some(c)) => {
            log(&format!(
                "Governor: {}, Token: {}, Timelock: {}, Delay: {}, Period: {}, Threshold: {}, Quorum: {}",
                c.name,
                c.governance_token,
                c.timelock_contract,
                c.voting_delay,
                c.voting_period,
                c.proposal_threshold,
                c.quorum_votes
            ));
            c
        }
        _ => {
            log("Failed to load governor config");
            return 0;
        }
    };
    1
}

/// Set governance token address (admin function)
#[unsafe(no_mangle)]
pub extern "C" fn set_governance_token(new_token: String) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is authorized (owner or admin)
    if let Err(_) = AccessControl::authorize(&ctx.sender(), Some("admin")) {
        log("Unauthorized: Only admin can set governance token");
        return;
    }

    let storage_ref = storage();
    let config: GovernorConfig = match storage_ref.get::<GovernorConfig>("config") {
        Ok(Some(mut c)) => {
            c.governance_token = new_token.clone();
            let mut storage_ref = storage();
            if storage_ref.set("config", &c).is_err() {
                log("Failed to update config");
                return;
            }
            log(&format!("Governance token updated to {}", new_token));
            c
        }
        _ => {
            log("Failed to load config");
            return;
        }
    };

    event!("GovernanceTokenUpdated", new_token: new_token);
}

/// Set timelock contract address (admin function)
#[unsafe(no_mangle)]
pub extern "C" fn set_timelock_contract(new_timelock: String) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is authorized (owner or admin)
    if let Err(_) = AccessControl::authorize(&ctx.sender(), Some("admin")) {
        log("Unauthorized: Only admin can set timelock contract");
        return;
    }

    let storage_ref = storage();
    let config: GovernorConfig = match storage_ref.get::<GovernorConfig>("config") {
        Ok(Some(mut c)) => {
            c.timelock_contract = new_timelock.clone();
            let mut storage_ref = storage();
            if storage_ref.set("config", &c).is_err() {
                log("Failed to update config");
                return;
            }
            log(&format!("Timelock contract updated to {}", new_timelock));
            c
        }
        _ => {
            log("Failed to load config");
            return;
        }
    };

    event!("TimelockContractUpdated", new_timelock: new_timelock);
}
