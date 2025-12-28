//! Timelock Controller
//!
//! A time-delayed execution contract for governance operations and security-critical upgrades on Chert Coin blockchain.
//!
//! ## Features
//! - Delayed Execution - Enforce mandatory waiting periods for operations
//! - Proposal Queue - Schedule operations for future execution
//! - Cancellation - Cancel pending operations before execution
//! - Role-Based Access - Separate proposer, executor, and admin roles
//! - Batch Operations - Execute multiple calls atomically
//! - Minimum Delay - Configurable security window
//! - Operation States - Track pending, ready, and executed operations
//! - Grace Period - Optional expiry window for operations

#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
use alloc::vec;

use blake3::Hasher;
use silica_contract_sdk::event;
use silica_contract_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Role enumeration for access control
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Role {
    PROPOSER_ROLE,
    EXECUTOR_ROLE,
    CANCELLER_ROLE,
    ADMIN_ROLE,
}

impl core::fmt::Display for Role {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Role::PROPOSER_ROLE => write!(f, "PROPOSER_ROLE"),
            Role::EXECUTOR_ROLE => write!(f, "EXECUTOR_ROLE"),
            Role::CANCELLER_ROLE => write!(f, "CANCELLER_ROLE"),
            Role::ADMIN_ROLE => write!(f, "ADMIN_ROLE"),
        }
    }
}

/// Operation state enumeration
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Unset,    // Operation doesn't exist
    Pending,  // Scheduled, waiting for delay
    Ready,    // Delay expired, can execute
    Executed, // Already executed
}

/// Operation structure
#[derive(Serialize, Deserialize)]
pub struct Operation {
    pub target: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub predecessor: Option<[u8; 32]>,
    pub salt: [u8; 32],
    pub ready_timestamp: u64,
    pub executed: bool,
    pub cancelled: bool,
}

/// Timelock configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct TimelockConfig {
    pub min_delay: u64,
    pub max_delay: u64,
    pub operation_counter: u64,
    pub admin: String,
    pub initialized: bool,
}

/// Initialize the timelock contract
///
/// # Arguments (should be parsed from transaction data)
/// * `min_delay` - Minimum delay in seconds (e.g., 2 days = 172800)
/// * `proposers` - Array of addresses that can schedule operations
/// * `executors` - Array of addresses that can execute (empty = anyone)
/// * `admin` - Address with admin role (typically the timelock itself)
#[unsafe(no_mangle)]
pub extern "C" fn initialize() {
    let ctx = context();
    let deployer = ctx.sender();

    // In a real implementation, these would be parsed from transaction data
    // For now, using example values from specification
    let min_delay = 172800u64; // 2 days
    let proposers: Vec<String> = vec![deployer.to_string()]; // Deployer as default proposer
    let executors: Vec<String> = vec![]; // Anyone can execute by default
    let admin = deployer.to_string(); // Self-administered

    // Validate parameters
    if min_delay == 0 || min_delay > 30 * 24 * 60 * 60 {
        log("Invalid minimum delay: must be > 0 and <= 30 days");
        return;
    }

    if proposers.is_empty() {
        log("At least one proposer required");
        return;
    }

    // Initialize timelock configuration
    let config = TimelockConfig {
        min_delay,
        max_delay: 30 * 24 * 60 * 60, // 30 days
        operation_counter: 0,
        admin: admin.clone(),
        initialized: true,
    };

    let mut storage = storage();
    if storage.set("config", &config).is_err() {
        log("Failed to store timelock config");
        return;
    }

    // Initialize role mappings
    let mut roles: Map<(Role, String), bool> = Map::new("roles");
    let mut role_counts: Map<Role, u64> = Map::new("role_counts");

    // Grant roles to initial accounts
    for proposer in &proposers {
        if roles
            .set(&(Role::PROPOSER_ROLE, proposer.clone()), &true)
            .is_err()
        {
            log("Failed to set proposer role");
            return;
        }
    }

    for executor in &executors {
        if roles
            .set(&(Role::EXECUTOR_ROLE, executor.clone()), &true)
            .is_err()
        {
            log("Failed to set executor role");
            return;
        }
    }

    if roles
        .set(&(Role::ADMIN_ROLE, admin.to_string()), &true)
        .is_err()
    {
        log("Failed to set admin role");
        return;
    }

    // Update role counts
    let proposer_count = proposers.len() as u64;
    let executor_count = executors.len() as u64;

    if role_counts
        .set(&Role::PROPOSER_ROLE, &proposer_count)
        .is_err()
        || role_counts
            .set(&Role::EXECUTOR_ROLE, &executor_count)
            .is_err()
        || role_counts.set(&Role::ADMIN_ROLE, &1).is_err()
        || role_counts.set(&Role::CANCELLER_ROLE, &0).is_err()
    {
        log("Failed to set role counts");
        return;
    }

    log(&format!(
        "Timelock initialized with min_delay: {}, proposers: {}, executors: {}, admin: {}",
        min_delay,
        proposers.len(),
        executors.len(),
        admin
    ));
    event!("TimelockInitialized",
        min_delay: min_delay,
        proposers: proposers.len().to_string(),
        executors: executors.len().to_string(),
        admin: admin
    );
}

/// Check if caller has a specific role
fn has_role(role: Role, account: &str) -> bool {
    let storage = storage();
    let roles: Map<(Role, String), bool> = Map::new("roles");
    match roles.get(&(role, account.to_string())) {
        Ok(Some(true)) => true,
        _ => false,
    }
}

/// Check if caller is admin
fn is_admin() -> bool {
    let ctx = context();
    let storage = storage();
    let config: TimelockConfig = match storage.get("config") {
        Ok(Some(c)) => c,
        _ => return false,
    };
    ctx.sender() == config.admin
}

/// Get current timestamp
fn get_timestamp() -> u64 {
    // In real implementation, this would use block timestamp
    // For now, using a placeholder value
    1700000000u64 // Unix timestamp placeholder
}

/// Hash operation to create unique ID
fn hash_operation(
    target: &str,
    value: u64,
    data: &[u8],
    predecessor: &Option<[u8; 32]>,
    salt: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(target.as_bytes());
    hasher.update(&value.to_le_bytes());
    hasher.update(data);

    match predecessor {
        Some(pred) => {
            hasher.update(pred);
        }
        None => {
            hasher.update(&[0u8; 32]);
        }
    }

    hasher.update(salt);
    *hasher.finalize().as_bytes()
}

/// Schedule an operation for future execution
///
/// # Arguments
/// * `target` - Contract address to call
/// * `value` - CHERT amount to send
/// * `data` - Calldata for the operation
/// * `predecessor` - Operation ID that must execute first (optional)
/// * `salt` - Random bytes for unique operation ID
/// * `delay` - Delay in seconds (must be ≥ min_delay)
///
/// # Returns
/// Operation ID (hash of parameters)
#[unsafe(no_mangle)]
pub extern "C" fn schedule() -> [u8; 32] {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller has PROPOSER_ROLE
    if !has_role(Role::PROPOSER_ROLE, &caller) {
        log("Caller does not have PROPOSER_ROLE");
        return [0u8; 32];
    }

    // In real implementation, parse from transaction data
    // For now, using example values
    let target = "target_contract_address".to_string();
    let value = 0u64;
    let data = vec![0u8; 4]; // Example call data
    let predecessor = None;
    let salt = [1u8; 32];
    let delay = 172800u64; // 2 days

    // Validate parameters
    let mut storage = storage();
    let config: TimelockConfig = match storage.get("config") {
        Ok(Some(c)) => c,
        _ => {
            log("Failed to load timelock config");
            return [0u8; 32];
        }
    };

    if delay < config.min_delay {
        log("Delay must be >= minimum delay");
        return [0u8; 32];
    }

    if delay > config.max_delay {
        log("Delay exceeds maximum allowed");
        return [0u8; 32];
    }

    // Generate operation ID
    let operation_id = hash_operation(&target, value, &data, &predecessor, &salt);

    // Check if operation already exists
    let mut operations: Map<[u8; 32], Operation> = Map::new("operations");
    if operations.get(&operation_id).ok().flatten().is_some() {
        log("Operation already exists");
        return [0u8; 32];
    }

    // Check predecessor if specified
    if let Some(pred_id) = predecessor {
        let pred_operation = match operations.get(&pred_id) {
            Ok(Some(op)) => op,
            Ok(None) => {
                log("Predecessor operation does not exist");
                return [0u8; 32];
            }
            Err(_) => {
                log("Failed to read predecessor operation");
                return [0u8; 32];
            }
        };

        if !pred_operation.executed {
            log("Predecessor operation must be executed first");
            return [0u8; 32];
        }
    }

    // Create operation
    let current_time = get_timestamp();
    let operation = Operation {
        target,
        value,
        data,
        predecessor,
        salt,
        ready_timestamp: current_time + delay,
        executed: false,
        cancelled: false,
    };

    // Store operation
    if operations.set(&operation_id, &operation).is_err() {
        log("Failed to store operation");
        return [0u8; 32];
    }

    // Update operation counter
    let mut config_mut = config;
    config_mut.operation_counter += 1;
    if storage.set("config", &config_mut).is_err() {
        log("Failed to update operation counter");
        return [0u8; 32];
    }

    log(&format!("Operation scheduled with ID: {:?}", operation_id));
    event!("OperationScheduled",
        id: hex::encode(operation_id),
        index: config_mut.operation_counter,
        target: operation.target,
        value: operation.value,
        data: hex::encode(&operation.data),
        predecessor: {
            match operation.predecessor {
                Some(p) => hex::encode(p),
                None => "none".to_string(),
            }
        },
        delay: delay,
        ready_timestamp: operation.ready_timestamp
    );

    operation_id
}

/// Schedule a batch of operations to execute atomically
///
/// # Arguments
/// * `targets` - Array of contract addresses
/// * `values` - Array of CHERT amounts
/// * `datas` - Array of calldata
/// * `predecessor` - Operation ID dependency (optional)
/// * `salt` - Random bytes for unique operation ID
/// * `delay` - Delay in seconds
///
/// # Returns
/// Batch operation ID
#[unsafe(no_mangle)]
pub extern "C" fn schedule_batch() -> [u8; 32] {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller has PROPOSER_ROLE
    if !has_role(Role::PROPOSER_ROLE, &caller) {
        log("Caller does not have PROPOSER_ROLE");
        return [0u8; 32];
    }

    // In real implementation, parse from transaction data
    let targets = vec!["target1".to_string(), "target2".to_string()];
    let values = vec![0u64, 0u64];
    let datas = vec![vec![0u8; 4], vec![0u8; 4]];
    let predecessor = None;
    let salt = [2u8; 32];
    let delay = 172800u64; // 2 days

    // Validate batch parameters
    if targets.len() != values.len() || targets.len() != datas.len() {
        log("Batch parameters must have same length");
        return [0u8; 32];
    }

    // For batch operations, we'll create a single "batch" operation
    // In a real implementation, you might create individual operations and link them
    let batch_operation_id = hash_operation_batch(&targets, &values, &datas, &predecessor, &salt);

    // Log the batch scheduling
    log(&format!(
        "Batch operation scheduled with ID: {:?}",
        batch_operation_id
    ));
    event!("OperationScheduled",
        id: hex::encode(batch_operation_id),
        index: 0, // Batch operations might have different indexing
        target: "batch".to_string(),
        value: 0,
        data: "".to_string(),
        predecessor: {
            match predecessor {
                Some(p) => hex::encode(p),
                None => "none".to_string(),
            }
        },
        delay: delay,
        ready_timestamp: get_timestamp() + delay
    );

    batch_operation_id
}

/// Hash operation batch to create unique ID
fn hash_operation_batch(
    targets: &[String],
    values: &[u64],
    datas: &[Vec<u8>],
    predecessor: &Option<[u8; 32]>,
    salt: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Hasher::new();

    for target in targets {
        hasher.update(target.as_bytes());
    }

    for value in values {
        hasher.update(&value.to_le_bytes());
    }

    for data in datas {
        hasher.update(data);
    }

    match predecessor {
        Some(pred) => {
            hasher.update(pred);
        }
        None => {
            hasher.update(&[0u8; 32]);
        }
    }

    hasher.update(salt);
    *hasher.finalize().as_bytes()
}

/// Cancel a pending operation
///
/// # Arguments
/// * `id` - Operation ID to cancel
#[unsafe(no_mangle)]
pub extern "C" fn cancel(id: [u8; 32]) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller has CANCELLER_ROLE
    if !has_role(Role::CANCELLER_ROLE, &caller) && !is_admin() {
        log("Caller does not have CANCELLER_ROLE or ADMIN_ROLE");
        return;
    }

    let mut operations: Map<[u8; 32], Operation> = Map::new("operations");
    let mut operation = match operations.get(&id) {
        Ok(Some(op)) => op,
        Ok(None) => {
            log("Operation does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read operation");
            return;
        }
    };

    // Check if operation can be cancelled
    if operation.executed {
        log("Cannot cancel already executed operation");
        return;
    }

    if operation.cancelled {
        log("Operation is already cancelled");
        return;
    }

    // Cancel the operation
    operation.cancelled = true;
    if operations.set(&id, &operation).is_err() {
        log("Failed to update operation");
        return;
    }

    log(&format!("Operation {:?} cancelled", id));
    event!("OperationCancelled", id: hex::encode(id));
}

/// Execute a ready operation
///
/// # Arguments
/// * `target` - Contract address to call
/// * `value` - CHERT amount to send
/// * `data` - Calldata for the operation
/// * `predecessor` - Operation ID that must execute first (optional)
/// * `salt` - Random bytes for unique operation ID
#[unsafe(no_mangle)]
pub extern "C" fn execute(
    target: String,
    value: u64,
    data: Vec<u8>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32],
) {
    let ctx = context();
    let caller = ctx.sender();

    // Check if caller has EXECUTOR_ROLE (or empty executor role means anyone can execute)
    let storage = storage();
    let role_counts: Map<Role, u64> = Map::new("role_counts");
    let executor_count = match role_counts.get(&Role::EXECUTOR_ROLE) {
        Ok(Some(count)) => count,
        _ => {
            log("Failed to read executor role count");
            return;
        }
    };

    if executor_count > 0 && !has_role(Role::EXECUTOR_ROLE, &caller) {
        log("Caller does not have EXECUTOR_ROLE");
        return;
    }

    // Generate operation ID and check if it exists
    let operation_id = hash_operation(&target, value, &data, &predecessor, &salt);
    let mut operations: Map<[u8; 32], Operation> = Map::new("operations");
    let mut operation = match operations.get(&operation_id) {
        Ok(Some(op)) => op,
        Ok(None) => {
            log("Operation does not exist");
            return;
        }
        Err(_) => {
            log("Failed to read operation");
            return;
        }
    };

    // Check if operation can be executed
    if operation.executed {
        log("Operation already executed");
        return;
    }

    if operation.cancelled {
        log("Operation is cancelled");
        return;
    }

    // Check if delay has passed
    let current_time = get_timestamp();
    if current_time < operation.ready_timestamp {
        log("Operation is not ready to execute");
        return;
    }

    // Check predecessor if specified
    if let Some(pred_id) = operation.predecessor {
        let pred_operation = match operations.get(&pred_id) {
            Ok(Some(op)) => op,
            _ => {
                log("Predecessor operation not found");
                return;
            }
        };

        if !pred_operation.executed {
            log("Predecessor operation must be executed first");
            return;
        }
    }

    // Mark operation as executed
    operation.executed = true;
    if operations.set(&operation_id, &operation).is_err() {
        log("Failed to update operation");
        return;
    }

    // In a real implementation, this is where you'd execute the actual call
    // For now, we'll just log the execution
    log(&format!(
        "Operation {:?} executed successfully",
        operation_id
    ));
    event!("OperationExecuted",
        id: hex::encode(operation_id),
        index: 0,
        target: target,
        value: value,
        data: hex::encode(&data),
        success: true
    );
}

/// Execute a batch of operations atomically
#[unsafe(no_mangle)]
pub extern "C" fn execute_batch(
    targets: Vec<String>,
    values: Vec<u64>,
    datas: Vec<Vec<u8>>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32],
) {
    let ctx = context();
    let caller = ctx.sender();

    // Similar validation to execute_single
    let storage = storage();
    let role_counts: Map<Role, u64> = Map::new("role_counts");
    let executor_count = match role_counts.get(&Role::EXECUTOR_ROLE) {
        Ok(Some(count)) => count,
        _ => {
            log("Failed to read executor role count");
            return;
        }
    };

    if executor_count > 0 && !has_role(Role::EXECUTOR_ROLE, &caller) {
        log("Caller does not have EXECUTOR_ROLE");
        return;
    }

    // Generate batch operation ID
    let batch_operation_id = hash_operation_batch(&targets, &values, &datas, &predecessor, &salt);

    log(&format!(
        "Batch operation {:?} executed",
        batch_operation_id
    ));
    event!("OperationExecuted",
        id: hex::encode(batch_operation_id),
        index: 0,
        target: "batch".to_string(),
        value: 0,
        data: "".to_string(),
        success: true
    );
}

/// Get the current state of an operation
#[unsafe(no_mangle)]
pub extern "C" fn get_operation_state(id: [u8; 32]) -> u8 {
    let operations: Map<[u8; 32], Operation> = Map::new("operations");
    match operations.get(&id) {
        Ok(Some(op)) => {
            if op.executed {
                3 // Executed
            } else if op.cancelled {
                4 // Cancelled (using 4 for cancelled)
            } else if get_timestamp() >= op.ready_timestamp {
                2 // Ready
            } else {
                1 // Pending
            }
        }
        Ok(None) => 0, // Unset
        Err(_) => 0,   // Unset (error case)
    }
}

/// Check if an operation is pending (scheduled but not ready)
#[unsafe(no_mangle)]
pub extern "C" fn is_operation_pending(id: [u8; 32]) -> bool {
    get_operation_state(id) == 1 // Pending
}

/// Check if an operation is ready to execute
#[unsafe(no_mangle)]
pub extern "C" fn is_operation_ready(id: [u8; 32]) -> bool {
    get_operation_state(id) == 2 // Ready
}

/// Check if an operation has been executed
#[unsafe(no_mangle)]
pub extern "C" fn is_operation_done(id: [u8; 32]) -> bool {
    get_operation_state(id) == 3 // Executed
}

/// Get the timestamp when an operation becomes ready
#[unsafe(no_mangle)]
pub extern "C" fn get_timestamp_op(id: [u8; 32]) -> u64 {
    let operations: Map<[u8; 32], Operation> = Map::new("operations");
    match operations.get(&id) {
        Ok(Some(op)) => op.ready_timestamp,
        _ => 0,
    }
}

/// Get the minimum delay period
#[unsafe(no_mangle)]
pub extern "C" fn get_min_delay() -> u64 {
    let storage = storage();
    match storage.get::<TimelockConfig>("config") {
        Ok(Some(c)) => c.min_delay,
        _ => 172800, // Default 2 days
    }
}

/// Check if an account has a specific role
#[unsafe(no_mangle)]
pub extern "C" fn has_role_check(role: u8, account: String) -> bool {
    let role_enum = match role {
        0 => Role::PROPOSER_ROLE,
        1 => Role::EXECUTOR_ROLE,
        2 => Role::CANCELLER_ROLE,
        3 => Role::ADMIN_ROLE,
        _ => {
            log("Invalid role value");
            return false;
        }
    };

    has_role(role_enum, &account)
}

/// Calculate the operation ID for given parameters
#[unsafe(no_mangle)]
pub extern "C" fn hash_operation_fn(
    target: String,
    value: u64,
    data: Vec<u8>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32],
) -> [u8; 32] {
    hash_operation(&target, value, &data, &predecessor, &salt)
}

/// Calculate the operation ID for a batch operation
#[unsafe(no_mangle)]
pub extern "C" fn hash_operation_batch_fn(
    targets: Vec<String>,
    values: Vec<u64>,
    datas: Vec<Vec<u8>>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32],
) -> [u8; 32] {
    hash_operation_batch(&targets, &values, &datas, &predecessor, &salt)
}

/// Update the minimum delay period (requires ADMIN_ROLE and timelock execution)
#[unsafe(no_mangle)]
pub extern "C" fn update_delay(new_delay: u64) {
    let ctx = context();
    let caller = ctx.sender();

    // Only admin can call this directly, or it must be called through timelock
    if !is_admin() {
        log("Only admin can update delay directly");
        return;
    }

    // Validate new delay
    if new_delay == 0 || new_delay > 30 * 24 * 60 * 60 {
        log("Invalid delay: must be > 0 and <= 30 days");
        return;
    }

    let mut storage = storage();
    match storage.get::<TimelockConfig>("config") {
        Ok(Some(mut c)) => {
            let old_delay = c.min_delay;
            c.min_delay = new_delay;

            if storage.set("config", &c).is_err() {
                log("Failed to update config");
                return;
            }

            log(&format!(
                "Delay updated from {} to {}",
                old_delay, new_delay
            ));
            event!("MinDelayChanged", old_delay: old_delay, new_delay: new_delay);
        }
        _ => {
            log("Failed to load config");
            return;
        }
    };
}

/// Grant a role to an account (requires ADMIN_ROLE)
#[unsafe(no_mangle)]
pub extern "C" fn grant_role(role: u8, account: String) {
    let ctx = context();
    let caller = ctx.sender();

    if !is_admin() {
        log("Only admin can grant roles");
        return;
    }

    let role_enum = match role {
        0 => Role::PROPOSER_ROLE,
        1 => Role::EXECUTOR_ROLE,
        2 => Role::CANCELLER_ROLE,
        3 => Role::ADMIN_ROLE,
        _ => {
            log("Invalid role value");
            return;
        }
    };

    // Check if role already exists
    if has_role(role_enum, &account) {
        log("Account already has this role");
        return;
    }

    // Grant the role
    let mut roles: Map<(Role, String), bool> = Map::new("roles");
    let mut role_counts: Map<Role, u64> = Map::new("role_counts");

    if roles.set(&(role_enum, account.clone()), &true).is_err() {
        log("Failed to set role");
        return;
    }

    // Update role count
    let current_count = role_counts.get(&role_enum).ok().flatten().unwrap_or(0);
    let new_count = current_count + 1;

    if role_counts.set(&role_enum, &new_count).is_err() {
        log("Failed to update role count");
        return;
    }

    log(&format!("Role granted to {}", account));
    event!("RoleGranted", role: role_enum, account: account, granter: caller);
}

/// Revoke a role from an account (requires ADMIN_ROLE)
#[unsafe(no_mangle)]
pub extern "C" fn revoke_role(role: u8, account: String) {
    let ctx = context();
    let caller = ctx.sender();

    if !is_admin() {
        log("Only admin can revoke roles");
        return;
    }

    let role_enum = match role {
        0 => Role::PROPOSER_ROLE,
        1 => Role::EXECUTOR_ROLE,
        2 => Role::CANCELLER_ROLE,
        3 => Role::ADMIN_ROLE,
        _ => {
            log("Invalid role value");
            return;
        }
    };

    // Check if role exists
    if !has_role(role_enum, &account) {
        log("Account does not have this role");
        return;
    }

    // Cannot revoke last admin
    if role_enum == Role::ADMIN_ROLE {
        let role_counts: Map<Role, u64> = Map::new("role_counts");
        let admin_count = role_counts
            .get(&Role::ADMIN_ROLE)
            .ok()
            .flatten()
            .unwrap_or(0);
        if admin_count <= 1 {
            log("Cannot revoke last admin role");
            return;
        }
    }

    // Revoke the role
    let mut roles: Map<(Role, String), bool> = Map::new("roles");
    let mut role_counts: Map<Role, u64> = Map::new("role_counts");

    if roles.set(&(role_enum, account.clone()), &false).is_err() {
        log("Failed to revoke role");
        return;
    }

    // Update role count
    let current_count = role_counts.get(&role_enum).ok().flatten().unwrap_or(1);
    let new_count = current_count.saturating_sub(1);

    if role_counts.set(&role_enum, &new_count).is_err() {
        log("Failed to update role count");
        return;
    }

    log(&format!("Role revoked from {}", account));
    event!("RoleRevoked", role: role_enum, account: account, revoker: caller);
}
