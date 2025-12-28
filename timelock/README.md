# Timelock Controller

A time-delayed execution contract for governance operations and security-critical upgrades on Chert Coin blockchain.

## Features

- ✅ **Delayed Execution** - Enforce mandatory waiting periods for operations
- ✅ **Proposal Queue** - Schedule operations for future execution
- ✅ **Cancellation** - Cancel pending operations before execution
- ✅ **Role-Based Access** - Separate proposer, executor, and admin roles
- ✅ **Batch Operations** - Execute multiple calls atomically
- ✅ **Minimum Delay** - Configurable security window
- ✅ **Operation States** - Track pending, ready, and executed operations
- ✅ **Grace Period** - Optional expiry window for operations

## Use Cases

- 🏛️ **DAO Governance** - Time-buffered proposal execution
- 🔐 **Protocol Upgrades** - Review period before contract changes
- 💼 **Treasury Operations** - Delayed spending for transparency
- 🛡️ **Emergency Response** - Cancellation window for malicious proposals
- ⚖️ **Legal Compliance** - Meet regulatory notice requirements
- 🔧 **Parameter Changes** - Staged deployment of system updates

## Architecture

### Operation Lifecycle

```
1. Scheduled  → Proposer schedules operation
2. Pending    → Waiting for delay period
3. Ready      → Delay expired, can execute
4. Executed   → Successfully executed
5. Cancelled  → Cancelled before execution (optional)
```

### Role Model

```
PROPOSER_ROLE:
- Can schedule operations
- Multiple addresses possible
- Typically: DAO governance contract

EXECUTOR_ROLE:
- Can execute ready operations
- Multiple addresses possible
- Can be address(0) = anyone can execute

CANCELLER_ROLE:
- Can cancel pending operations
- Emergency security role
- Typically: Security council

ADMIN_ROLE:
- Can grant/revoke roles
- Can change minimum delay
- Highest privilege level
- Typically: The timelock itself (self-governed)
```

## API Reference

### Initialize

```rust
fn initialize(
    min_delay: u64,
    proposers: Vec<String>,
    executors: Vec<String>,
    admin: String
)
```

Initializes the timelock controller.

**Parameters:**
- `min_delay` - Minimum delay in seconds (e.g., 2 days = 172800)
- `proposers` - Array of addresses that can schedule operations
- `executors` - Array of addresses that can execute (empty = anyone)
- `admin` - Address with admin role (typically the timelock itself)

**Requirements:**
- Min delay must be reasonable (< 30 days)
- At least one proposer
- Admin address must be valid

**Events:**
- `TimelockInitialized { min_delay, proposers, executors, admin }`

### Schedule Operation

```rust
fn schedule(
    target: String,
    value: u64,
    data: Vec<u8>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32],
    delay: u64
) -> [u8; 32]
```

Schedules an operation for future execution.

**Parameters:**
- `target` - Contract address to call
- `value` - CHERT amount to send
- `data` - Calldata for the operation
- `predecessor` - Operation ID that must execute first (optional)
- `salt` - Random bytes for unique operation ID
- `delay` - Delay in seconds (must be ≥ min_delay)

**Returns:** Operation ID (hash of parameters)

**Requirements:**
- Caller must have PROPOSER_ROLE
- Delay must be ≥ minimum delay
- Operation ID must not already exist
- If predecessor specified, it must be executed

**Events:**
- `OperationScheduled { id, target, value, delay, predecessor }`

### Schedule Batch

```rust
fn schedule_batch(
    targets: Vec<String>,
    values: Vec<u64>,
    datas: Vec<Vec<u8>>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32],
    delay: u64
) -> [u8; 32]
```

Schedules multiple operations to execute atomically.

**Parameters:**
- `targets` - Array of contract addresses
- `values` - Array of CHERT amounts
- `datas` - Array of calldata
- `predecessor` - Operation ID dependency (optional)
- `salt` - Random bytes for unique operation ID
- `delay` - Delay in seconds

**Returns:** Operation ID

**Requirements:**
- All arrays must have same length
- Caller must have PROPOSER_ROLE
- Delay must be ≥ minimum delay

**Events:**
- `OperationScheduled` for the batch

### Cancel Operation

```rust
fn cancel(id: [u8; 32])
```

Cancels a pending operation.

**Parameters:**
- `id` - Operation ID to cancel

**Requirements:**
- Caller must have CANCELLER_ROLE
- Operation must be pending (not executed)

**Events:**
- `OperationCancelled { id }`

### Execute Operation

```rust
fn execute(
    target: String,
    value: u64,
    data: Vec<u8>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32]
)
```

Executes a ready operation.

**Parameters:**
- Must match exactly what was scheduled
- `target`, `value`, `data`, `predecessor`, `salt` - Same as schedule

**Requirements:**
- Caller must have EXECUTOR_ROLE (or anyone if EXECUTOR_ROLE is empty)
- Operation must be ready (delay expired)
- If predecessor specified, it must be executed
- Not already executed

**Events:**
- `OperationExecuted { id, target, value, success }`

### Execute Batch

```rust
fn execute_batch(
    targets: Vec<String>,
    values: Vec<u64>,
    datas: Vec<Vec<u8>>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32]
)
```

Executes a batch of operations atomically.

**Parameters:**
- Must match exactly what was scheduled

**Requirements:**
- Same as execute
- All operations execute or all revert

**Events:**
- `OperationExecuted` for the batch

### Update Delay

```rust
fn update_delay(new_delay: u64)
```

Changes the minimum delay period.

**Parameters:**
- `new_delay` - New minimum delay in seconds

**Requirements:**
- Must be called through timelock (schedule -> execute)
- New delay must be reasonable (< 30 days)

**Events:**
- `MinDelayChanged { old_delay, new_delay }`

### Grant Role

```rust
fn grant_role(role: Role, account: String)
```

Grants a role to an account.

**Parameters:**
- `role` - Role to grant (PROPOSER/EXECUTOR/CANCELLER/ADMIN)
- `account` - Address to grant role to

**Requirements:**
- Must be called through timelock (schedule -> execute)
- Or called by current admin

**Events:**
- `RoleGranted { role, account, granter }`

### Revoke Role

```rust
fn revoke_role(role: Role, account: String)
```

Revokes a role from an account.

**Parameters:**
- `role` - Role to revoke
- `account` - Address to revoke role from

**Requirements:**
- Must be called through timelock
- Cannot revoke last proposer

**Events:**
- `RoleRevoked { role, account, revoker }`

## Query Functions

### Get Operation State

```rust
fn get_operation_state(id: [u8; 32]) -> OperationState
```

Returns the current state of an operation.

**Returns:** `Unset | Pending | Ready | Executed`

```rust
enum OperationState {
    Unset,      // Operation doesn't exist
    Pending,    // Scheduled, waiting for delay
    Ready,      // Delay expired, can execute
    Executed,   // Already executed
}
```

### Is Operation Pending

```rust
fn is_operation_pending(id: [u8; 32]) -> bool
```

Checks if an operation is scheduled but not ready.

**Returns:** True if pending

### Is Operation Ready

```rust
fn is_operation_ready(id: [u8; 32]) -> bool
```

Checks if an operation can be executed.

**Returns:** True if ready

### Is Operation Done

```rust
fn is_operation_done(id: [u8; 32]) -> bool
```

Checks if an operation has been executed.

**Returns:** True if executed

### Get Operation Timestamp

```rust
fn get_timestamp(id: [u8; 32]) -> u64
```

Returns when an operation becomes ready.

**Returns:** Unix timestamp

### Get Minimum Delay

```rust
fn get_min_delay() -> u64
```

Returns the current minimum delay period.

**Returns:** Delay in seconds

### Has Role

```rust
fn has_role(role: Role, account: String) -> bool
```

Checks if an account has a specific role.

**Returns:** True if account has role

### Hash Operation

```rust
fn hash_operation(
    target: String,
    value: u64,
    data: Vec<u8>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32]
) -> [u8; 32]
```

Calculates the operation ID for given parameters.

**Returns:** Operation ID hash

### Hash Operation Batch

```rust
fn hash_operation_batch(
    targets: Vec<String>,
    values: Vec<u64>,
    datas: Vec<Vec<u8>>,
    predecessor: Option<[u8; 32]>,
    salt: [u8; 32]
) -> [u8; 32]
```

Calculates the operation ID for a batch.

**Returns:** Batch operation ID

## Events

```rust
// Emitted when timelock is initialized
event TimelockInitialized {
    min_delay: u64,
    proposers: Vec<String>,
    executors: Vec<String>,
    admin: String,
}

// Emitted when operation is scheduled
event OperationScheduled {
    id: [u8; 32],
    index: u64,
    target: String,
    value: u64,
    data: Vec<u8>,
    predecessor: Option<[u8; 32]>,
    delay: u64,
    ready_timestamp: u64,
}

// Emitted when operation is cancelled
event OperationCancelled {
    id: [u8; 32],
}

// Emitted when operation is executed
event OperationExecuted {
    id: [u8; 32],
    index: u64,
    target: String,
    value: u64,
    data: Vec<u8>,
    success: bool,
}

// Emitted when minimum delay changes
event MinDelayChanged {
    old_delay: u64,
    new_delay: u64,
}

// Emitted when role is granted
event RoleGranted {
    role: Role,
    account: String,
    granter: String,
}

// Emitted when role is revoked
event RoleRevoked {
    role: Role,
    account: String,
    revoker: String,
}
```

## Storage Layout

```rust
// Core state
u64: "min_delay"
u64: "operation_counter"

// Operation tracking
Map<[u8; 32], u64>: "timestamps"  // id -> ready_timestamp
Map<[u8; 32], bool>: "executed"   // id -> executed

// Role-based access control
Map<(Role, String), bool>: "roles"  // (role, account) -> has_role
Map<Role, u64>: "role_counts"       // role -> member_count

// Operation dependencies
Map<[u8; 32], Option<[u8; 32]>>: "predecessors"  // id -> predecessor_id

// Configuration
u64: "max_delay"  // 30 days
```

## Security Considerations

### Timing Attacks Prevention
- Timestamps use block time, not transaction time
- Minimum delay enforced before any execution
- No early execution allowed

### Role Security
- Admin role should be timelock itself (self-governed)
- Separate proposer and executor roles
- Canceller role for emergency response
- Role changes require timelock delay

### Operation Security
- Operation ID includes salt for uniqueness
- Parameters must match exactly for execution
- Predecessor dependencies prevent out-of-order execution
- Reentrancy protection on execution

### Execution Safety
- External call failures don't revert entire transaction
- Success/failure tracked in events
- Value transfers validated before execution
- Batch operations are atomic (all succeed or all fail)

### Integer Safety
- Delay overflow protection
- Timestamp arithmetic bounds checking
- Maximum delay limits (30 days)

## Example Usage

### Deploying DAO Timelock

```rust
// Deploy timelock with 2-day delay
let timelock = deploy_timelock();

timelock.initialize(
    172800,  // 2 days in seconds
    vec![dao_governor_address],  // Only DAO can propose
    vec![],  // Anyone can execute
    timelock_address  // Self-administered
);
```

### Scheduling a Protocol Upgrade

```rust
// Schedule upgrade with 3-day delay
let operation_id = timelock.schedule(
    protocol_contract,
    0,  // No value
    encode_call("upgrade_to", new_implementation),
    None,  // No predecessor
    random_salt(),
    259200  // 3 days
);

// Wait 3 days...
// Anyone can execute after delay
timelock.execute(
    protocol_contract,
    0,
    encode_call("upgrade_to", new_implementation),
    None,
    salt
);
```

### Batch Treasury Operations

```rust
// Schedule multiple token transfers atomically
let targets = vec![token1, token2, token3];
let values = vec![0, 0, 0];
let datas = vec![
    encode_call("transfer", recipient1, 1000),
    encode_call("transfer", recipient2, 2000),
    encode_call("transfer", recipient3, 3000),
];

let operation_id = timelock.schedule_batch(
    targets,
    values,
    datas,
    None,
    random_salt(),
    172800  // 2 days
);

// Execute all transfers atomically after delay
timelock.execute_batch(targets, values, datas, None, salt);
```

### Emergency Cancellation

```rust
// Security council detects malicious proposal
timelock.cancel(suspicious_operation_id);
// Operation cannot be executed
```

### Changing Minimum Delay

```rust
// Increase security by extending delay
let operation_id = timelock.schedule(
    timelock_address,  // Call self
    0,
    encode_call("update_delay", 604800),  // Increase to 7 days
    None,
    random_salt(),
    current_min_delay  // Use current delay for this change
);

// Execute after current delay
timelock.execute(
    timelock_address,
    0,
    encode_call("update_delay", 604800),
    None,
    salt
);
// Future operations require 7-day delay
```

### Operation Dependencies

```rust
// Step 1: Pause protocol
let pause_id = timelock.schedule(
    protocol,
    0,
    encode_call("pause"),
    None,
    salt1,
    172800
);

// Step 2: Upgrade (depends on pause)
let upgrade_id = timelock.schedule(
    protocol,
    0,
    encode_call("upgrade"),
    Some(pause_id),  // Must pause first
    salt2,
    172800
);

// Step 3: Unpause (depends on upgrade)
let unpause_id = timelock.schedule(
    protocol,
    0,
    encode_call("unpause"),
    Some(upgrade_id),  // Must upgrade first
    salt3,
    172800
);

// Must execute in order: pause -> upgrade -> unpause
```

## Integration with Governance

### DAO Governor + Timelock

```rust
// Deploy governor with timelock executor
let governor = deploy_governor(
    governance_token,
    timelock_address,  // Timelock controls execution
    voting_delay,
    voting_period,
    quorum
);

// Grant proposer role to governor
timelock.grant_role(PROPOSER_ROLE, governor_address);

// Flow: Vote passes -> Governor schedules on timelock -> Wait -> Execute
```

## Differences from OpenZeppelin TimelockController

- ✅ **Native Integration** - Built for Chert's architecture
- ✅ **Simpler Interface** - Focused on core timelock functionality
- ✅ **Lower Gas Costs** - Optimized storage layout
- ✅ **Post-Quantum Ready** - Compatible with Dilithium signatures

## Testing Checklist

- [ ] Initialize with various delays and roles
- [ ] Schedule operations with minimum delay
- [ ] Schedule operations with longer delays
- [ ] Execute operations after delay expires
- [ ] Fail execution before delay expires
- [ ] Cancel pending operations
- [ ] Schedule batch operations
- [ ] Execute batch operations atomically
- [ ] Test operation dependencies (predecessors)
- [ ] Grant and revoke roles via timelock
- [ ] Change minimum delay via timelock
- [ ] Test emergency cancellation
- [ ] Verify role-based access control
- [ ] Test reentrancy protection
- [ ] Test with maximum operations queue

## License

MIT License - See LICENSE file for details

## References

- [OpenZeppelin TimelockController](https://docs.openzeppelin.com/contracts/api/governance#TimelockController)
- [Compound Timelock](https://github.com/compound-finance/compound-protocol/blob/master/contracts/Timelock.sol)
- [Governance Best Practices](https://blog.openzeppelin.com/govern-smart-contracts-security)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q1 2026
