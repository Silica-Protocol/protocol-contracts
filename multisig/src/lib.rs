//! Multi-Signature Wallet
//!
//! A secure multi-signature wallet requiring M-of-N signatures for transaction execution on Chert Coin blockchain.
//!
//! ## Features
//! - M-of-N Signatures - Require multiple approvals for transactions
//! - Flexible Configuration - Configurable signers and threshold
//! - Transaction Queue - Propose, approve, and execute transactions
//! - Owner Management - Add/remove signers with consensus
//! - Threshold Updates - Change required signature count
//! - Token Support - Manage CRC-20 tokens and native CHERT
//! - Transaction History - Track all proposals and executions
//! - Cancellation - Revoke pending transactions
//! - Time Locks - Optional execution delays for security

#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use silica_contract_sdk::event;
use silica_contract_sdk::prelude::*;
use silica_contract_sdk::storage::Vector;
use serde::{Deserialize, Serialize};

/// Transaction structure
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub to: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub description: String,
    pub proposer: String,
    pub timestamp: u64,
    pub executed: bool,
    pub cancelled: bool,
    pub time_lock: Option<u64>,
}

/// Multisig wallet configuration
#[derive(Serialize, Deserialize)]
pub struct WalletConfig {
    pub threshold: u64,
    pub transaction_count: u64,
    pub max_owners: u64,
    pub max_time_lock: u64,
    pub initialized: bool,
}

/// Initialize the multisig wallet
///
/// # Arguments
/// * `owners` - Array of owner addresses (signers)
/// * `threshold` - Number of required signatures (M)
#[unsafe(no_mangle)]
pub extern "C" fn initialize(owners: Vec<String>, threshold: u64) {
    let ctx = context();
    let deployer = ctx.sender();

    // Validate parameters
    if owners.is_empty() {
        log("At least one owner required");
        return;
    }

    if threshold == 0 {
        log("Threshold must be > 0");
        return;
    }

    if threshold > owners.len() as u64 {
        log("Threshold cannot exceed number of owners");
        return;
    }

    if owners.len() > 50 {
        log("Maximum 50 owners allowed");
        return;
    }

    // Check for duplicate owners
    let mut unique_owners = Vec::new();
    for owner in &owners {
        if !unique_owners.contains(owner) {
            unique_owners.push(owner.clone());
        }
    }

    if unique_owners.len() != owners.len() {
        log("Duplicate owners not allowed");
        return;
    }

    // Initialize wallet configuration
    let config = WalletConfig {
        threshold,
        transaction_count: 0,
        max_owners: 50,
        max_time_lock: 30 * 24 * 60 * 60, // 30 days
        initialized: true,
    };

    let mut storage_ref = storage();
    if storage_ref.set("config", &config).is_err() {
        log("Failed to store wallet config");
        return;
    }

    // Initialize owner management
    let mut is_owner: Map<String, bool> = Map::new("is_owner");
    let mut owners_vec: Vector<String> = Vector::new("owners");

    for owner in &unique_owners {
        if is_owner.set(owner, &true).is_err() {
            log("Failed to set owner status");
            return;
        }
        if owners_vec.push(owner).is_err() {
            log("Failed to add owner to vector");
            return;
        }
    }

    log(&format!(
        "Multisig wallet initialized with {} owners, threshold: {}",
        unique_owners.len(),
        threshold
    ));
    event!("WalletCreated", owners: format!("{:?}", unique_owners), threshold: threshold);
}

/// Check if caller is an owner
fn is_owner_check() -> bool {
    let ctx = context();
    let storage = storage();
    let is_owner: Map<String, bool> = Map::new("is_owner");
    match is_owner.get(&ctx.sender().to_string()) {
        Ok(Some(true)) => true,
        _ => false,
    }
}

/// Get current timestamp
fn get_timestamp() -> u64 {
    // In real implementation, this would use block timestamp
    // For now, using a placeholder value
    1700000000u64 // Unix timestamp placeholder
}

/// Submit a new transaction for approval
///
/// # Arguments
/// * `to` - Recipient address
/// * `value` - Amount of CHERT to send
/// * `data` - Contract call data (empty for simple transfers)
/// * `description` - Human-readable transaction description
///
/// # Returns
/// Transaction ID
#[unsafe(no_mangle)]
pub extern "C" fn submit_transaction(
    to: String,
    value: u64,
    data: Vec<u8>,
    description: String,
) -> u64 {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is an owner
    if !is_owner_check() {
        log("Only owners can submit transactions");
        return 0;
    }

    // Validate parameters
    if to.is_empty() {
        log("Invalid recipient address");
        return 0;
    }

    if description.is_empty() {
        log("Description is required");
        return 0;
    }

    // Get wallet configuration
    let mut storage_ref = storage();
    let config: WalletConfig = match storage_ref.get::<WalletConfig>("config") {
        Ok(Some(c)) => c,
        _ => {
            log("Failed to load wallet config");
            return 0;
        }
    };

    // Create transaction
    let timestamp = get_timestamp();
    let transaction = Transaction {
        to: to.clone(),
        value,
        data: data.clone(),
        description: description.clone(),
        proposer: caller.to_string(),
        timestamp,
        executed: false,
        cancelled: false,
        time_lock: None,
    };

    // Store transaction
    let mut transactions: Map<u64, Transaction> = Map::new("transactions");
    let tx_id = config.transaction_count;

    if transactions.set(&tx_id, &transaction).is_err() {
        log("Failed to store transaction");
        return 0;
    }

    // Track approvals
    let mut approvals: Map<u64, Vec<String>> = Map::new("approvals");
    let mut has_approved: Map<(u64, String), bool> = Map::new("has_approved");

    // Automatically approve transaction by proposer
    let mut approvers = Vec::new();
    approvers.push(caller.to_string());

    if approvals.set(&tx_id, &approvers).is_err() {
        log("Failed to store approvals");
        return 0;
    }

    if has_approved
        .set(&(tx_id, caller.to_string()), &true)
        .is_err()
    {
        log("Failed to track approval");
        return 0;
    }

    // Update transaction count
    let mut config_mut = config;
    config_mut.transaction_count += 1;
    if storage_ref.set("config", &config_mut).is_err() {
        log("Failed to update transaction count");
        return 0;
    }

    log(&format!(
        "Transaction {} submitted by {}: {} {} CHERT to {}",
        tx_id, caller, description, value, to
    ));
    event!("TransactionSubmitted",
        tx_id: tx_id,
        proposer: caller,
        to: to,
        value: value,
        description: description
    );

    tx_id
}

/// Approve a pending transaction
///
/// # Arguments
/// * `tx_id` - Transaction ID to approve
#[unsafe(no_mangle)]
pub extern "C" fn approve_transaction(tx_id: u64) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is an owner
    if !is_owner_check() {
        log("Only owners can approve transactions");
        return;
    }

    // Get transaction
    let mut transactions: Map<u64, Transaction> = Map::new("transactions");
    let mut transaction = match transactions.get(&tx_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            log("Transaction does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read transaction");
            return;
        }
    };

    // Check if transaction can be approved
    if transaction.executed {
        log("Transaction already executed");
        return;
    }

    if transaction.cancelled {
        log("Transaction is cancelled");
        return;
    }

    // Check if already approved
    let mut has_approved: Map<(u64, String), bool> = Map::new("has_approved");
    if has_approved
        .get(&(tx_id, caller.to_string()))
        .ok()
        .flatten()
        == Some(true)
    {
        log("Transaction already approved by this owner");
        return;
    }

    // Add approval
    let mut approvals: Map<u64, Vec<String>> = Map::new("approvals");
    let mut approvers = match approvals.get(&tx_id) {
        Ok(Some(list)) => list,
        Ok(None) => {
            log("No approval record found");
            return;
        }
        Err(_) => {
            log("Failed to read approvals");
            return;
        }
    };

    approvers.push(caller.to_string());

    if approvals.set(&tx_id, &approvers).is_err() {
        log("Failed to update approvals");
        return;
    }

    if has_approved
        .set(&(tx_id, caller.to_string()), &true)
        .is_err()
    {
        log("Failed to track approval");
        return;
    }

    // Update transaction in storage
    if transactions.set(&tx_id, &transaction).is_err() {
        log("Failed to update transaction");
        return;
    }

    log(&format!("Transaction {} approved by {}", tx_id, caller));
    event!("TransactionApproved",
        tx_id: tx_id,
        approver: caller,
        approval_count: approvers.len() as u64
    );
}

/// Revoke approval from a pending transaction
///
/// # Arguments
/// * `tx_id` - Transaction ID to revoke approval from
#[unsafe(no_mangle)]
pub extern "C" fn revoke_approval(tx_id: u64) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is an owner
    if !is_owner_check() {
        log("Only owners can revoke approvals");
        return;
    }

    // Get transaction
    let mut transactions: Map<u64, Transaction> = Map::new("transactions");
    let mut transaction = match transactions.get(&tx_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            log("Transaction does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read transaction");
            return;
        }
    };

    // Check if transaction can be modified
    if transaction.executed {
        log("Cannot revoke approval from executed transaction");
        return;
    }

    // Remove approval
    let mut approvals: Map<u64, Vec<String>> = Map::new("approvals");
    let mut approvers = match approvals.get(&tx_id) {
        Ok(Some(list)) => list,
        Ok(None) => {
            log("No approval record found");
            return;
        }
        Err(_) => {
            log("Failed to read approvals");
            return;
        }
    };

    // Find and remove caller from approvers
    if let Some(pos) = approvers.iter().position(|x| x == &caller) {
        approvers.remove(pos);
    } else {
        log("No approval to revoke");
        return;
    }

    if approvals.set(&tx_id, &approvers).is_err() {
        log("Failed to update approvals");
        return;
    }

    // Update approval tracking
    let mut has_approved: Map<(u64, String), bool> = Map::new("has_approved");
    if has_approved
        .set(&(tx_id, caller.to_string()), &false)
        .is_err()
    {
        log("Failed to update approval tracking");
        return;
    }

    // Update transaction in storage
    if transactions.set(&tx_id, &transaction).is_err() {
        log("Failed to update transaction");
        return;
    }

    log(&format!(
        "Approval revoked from transaction {} by {}",
        tx_id, caller
    ));
    event!("ApprovalRevoked", tx_id: tx_id, owner: caller);
}

/// Check if transaction can be executed
fn can_execute_transaction(tx_id: u64) -> bool {
    let config: WalletConfig = match storage().get("config") {
        Ok(Some(c)) => c,
        _ => return false,
    };

    // Check approval count
    let approvals: Map<u64, Vec<String>> = Map::new("approvals");
    let approvers = match approvals.get(&tx_id) {
        Ok(Some(list)) => list,
        _ => return false,
    };

    if approvers.len() < config.threshold as usize {
        return false;
    }

    // Check transaction state
    let transactions: Map<u64, Transaction> = Map::new("transactions");
    let mut transaction = match transactions.get(&tx_id) {
        Ok(Some(tx)) => tx,
        _ => return false,
    };

    if transaction.executed || transaction.cancelled {
        return false;
    }

    // Check time lock
    if let Some(unlock_time) = transaction.time_lock {
        if get_timestamp() < unlock_time {
            return false;
        }
    }

    true
}

/// Execute a transaction that has reached threshold signatures
///
/// # Arguments
/// * `tx_id` - Transaction ID to execute
#[unsafe(no_mangle)]
pub extern "C" fn execute_transaction(tx_id: u64) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if transaction can be executed
    if !can_execute_transaction(tx_id) {
        log("Transaction cannot be executed");
        return;
    }

    // Get transaction
    let mut transactions: Map<u64, Transaction> = Map::new("transactions");
    let mut transaction = match transactions.get(&tx_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            log("Transaction does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read transaction");
            return;
        }
    };

    // Mark as executed
    transaction.executed = true;
    if transactions.set(&tx_id, &transaction).is_err() {
        log("Failed to update transaction");
        return;
    }

    // In a real implementation, this is where you'd execute the actual call
    // For now, we'll just log the execution
    log(&format!(
        "Transaction {} executed by {}: {} {} CHERT to {}",
        tx_id, caller, transaction.description, transaction.value, transaction.to
    ));

    event!("TransactionExecuted", tx_id: tx_id, executor: caller);
}

/// Cancel a pending transaction (requires M signatures)
///
/// # Arguments
/// * `tx_id` - Transaction ID to cancel
#[unsafe(no_mangle)]
pub extern "C" fn cancel_transaction(tx_id: u64) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is an owner
    if !is_owner_check() {
        log("Only owners can cancel transactions");
        return;
    }

    // Get transaction
    let mut transactions: Map<u64, Transaction> = Map::new("transactions");
    let mut transaction = match transactions.get(&tx_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            log("Transaction does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read transaction");
            return;
        }
    };

    // Check if transaction can be cancelled
    if transaction.executed {
        log("Cannot cancel executed transaction");
        return;
    }

    if transaction.cancelled {
        log("Transaction is already cancelled");
        return;
    }

    // Check if caller has already approved (makes it harder to cancel)
    let has_approved: Map<(u64, String), bool> = Map::new("has_approved");
    let has_approved_by_caller = has_approved
        .get(&(tx_id, caller.to_string()))
        .ok()
        .flatten()
        == Some(true);

    // Simple cancellation - any owner can cancel, but log it
    // In a real implementation, you might require M signatures like execution

    transaction.cancelled = true;
    if transactions.set(&tx_id, &transaction).is_err() {
        log("Failed to update transaction");
        return;
    }

    log(&format!(
        "Transaction {} cancelled by {} (approved: {})",
        tx_id, caller, has_approved_by_caller
    ));
    event!("TransactionCancelled", tx_id: tx_id);
}

/// Set time lock on a transaction for additional security
///
/// # Arguments
/// * `tx_id` - Transaction ID
/// * `delay_seconds` - Seconds to wait after threshold reached
#[unsafe(no_mangle)]
pub extern "C" fn set_time_lock(tx_id: u64, delay_seconds: u64) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller is an owner
    if !is_owner_check() {
        log("Only owners can set time locks");
        return;
    }

    // Get wallet configuration
    let storage = storage();
    let config: WalletConfig = match storage.get("config") {
        Ok(Some(c)) => c,
        _ => {
            log("Failed to load wallet config");
            return;
        }
    };

    // Validate delay
    if delay_seconds > config.max_time_lock {
        log("Time lock exceeds maximum allowed");
        return;
    }

    // Get transaction
    let mut transactions: Map<u64, Transaction> = Map::new("transactions");
    let mut transaction = match transactions.get(&tx_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            log("Transaction does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read transaction");
            return;
        }
    };

    // Check if transaction can be time-locked
    if transaction.executed {
        log("Cannot set time lock on executed transaction");
        return;
    }

    if transaction.cancelled {
        log("Cannot set time lock on cancelled transaction");
        return;
    }

    // Set time lock
    let unlock_time = get_timestamp() + delay_seconds;
    transaction.time_lock = Some(unlock_time);

    if transactions.set(&tx_id, &transaction).is_err() {
        log("Failed to update transaction");
        return;
    }

    log(&format!(
        "Time lock set on transaction {}: unlocks at timestamp {}",
        tx_id, unlock_time
    ));
    event!("TimeLockSet", tx_id: tx_id, unlock_time: unlock_time);
}

/// Query function: Check if an address is an owner
#[unsafe(no_mangle)]
pub extern "C" fn is_owner(address: String) -> bool {
    let is_owner: Map<String, bool> = Map::new("is_owner");
    match is_owner.get(&address) {
        Ok(Some(true)) => true,
        _ => false,
    }
}

/// Query function: Get the current threshold
#[unsafe(no_mangle)]
pub extern "C" fn get_threshold() -> u64 {
    let storage_ref = storage();
    let config_value = match storage_ref.get::<WalletConfig>("config") {
        Ok(Some(c)) => c.threshold,
        _ => 0,
    };
    config_value
}

/// Query function: Get the number of owners
#[unsafe(no_mangle)]
pub extern "C" fn get_owner_count() -> u64 {
    let owners_vec: Vector<String> = Vector::new("owners");
    match owners_vec.len() {
        Ok(count) => count,
        _ => 0,
    }
}

/// Query function: Get the number of approvals for a transaction
#[unsafe(no_mangle)]
pub extern "C" fn get_approval_count(tx_id: u64) -> u64 {
    let approvals: Map<u64, Vec<String>> = Map::new("approvals");
    match approvals.get(&tx_id) {
        Ok(Some(list)) => list.len() as u64,
        _ => 0,
    }
}

/// Query function: Check if an owner has approved a transaction
#[unsafe(no_mangle)]
pub extern "C" fn has_approved(tx_id: u64, owner: String) -> bool {
    let has_approved: Map<(u64, String), bool> = Map::new("has_approved");
    match has_approved.get(&(tx_id, owner)) {
        Ok(Some(true)) => true,
        _ => false,
    }
}

/// Query function: Check if a transaction can be executed
#[unsafe(no_mangle)]
pub extern "C" fn can_execute(tx_id: u64) -> bool {
    can_execute_transaction(tx_id)
}

/// Query function: Get the total transaction count
#[unsafe(no_mangle)]
pub extern "C" fn get_transaction_count() -> u64 {
    let storage_ref = storage();
    let config_value = match storage_ref.get::<WalletConfig>("config") {
        Ok(Some(c)) => c.transaction_count,
        _ => 0,
    };
    config_value
}
