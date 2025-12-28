# Multi-Signature Wallet

A secure multi-signature wallet requiring M-of-N signatures for transaction execution on Chert Coin blockchain.

## Features

- ✅ **M-of-N Signatures** - Require multiple approvals for transactions
- ✅ **Flexible Configuration** - Configurable signers and threshold
- ✅ **Transaction Queue** - Propose, approve, and execute transactions
- ✅ **Owner Management** - Add/remove signers with consensus
- ✅ **Threshold Updates** - Change required signature count
- ✅ **Token Support** - Manage CRC-20 tokens and native CHERT
- ✅ **Transaction History** - Track all proposals and executions
- ✅ **Cancellation** - Revoke pending transactions
- ✅ **Time Locks** - Optional execution delays for security

## Use Cases

- 🏢 **Corporate Treasury** - Require multiple executives to approve spending
- 🤝 **Joint Accounts** - Shared wallets for partnerships or families
- 🏦 **DAO Treasury** - Multi-party control over community funds
- 🔐 **Cold Storage** - Distributed key security for large holdings
- 💼 **Escrow Services** - Third-party transaction mediation
- 🎯 **Project Funds** - Team-controlled project budgets

## Architecture

### Signature Model

```
M-of-N Multisig:
- N = Total number of signers/owners
- M = Required signatures (threshold)
- M ≤ N
- Common configurations: 2/3, 3/5, 5/7

Example: 3-of-5 multisig
- 5 owners can propose transactions
- 3 signatures required to execute
- Any 3 of the 5 can approve
```

### Transaction Flow

```
1. Propose → Any owner submits transaction
2. Approve → M owners sign the transaction
3. Execute → Anyone calls execute after M signatures
4. Complete → Transaction executes on blockchain
```

## API Reference

### Initialize

```rust
fn initialize(owners: Vec<String>, threshold: u64)
```

Creates a new multisig wallet with initial configuration.

**Parameters:**
- `owners` - Array of owner addresses (signers)
- `threshold` - Number of required signatures (M)

**Requirements:**
- Threshold > 0
- Threshold ≤ owners.length
- Owners must be unique addresses
- Minimum 1 owner
- Maximum 50 owners (gas limit protection)

**Events:**
- `WalletCreated { owners, threshold }`

### Submit Transaction

```rust
fn submit_transaction(
    to: String,
    value: u64,
    data: Vec<u8>,
    description: String
) -> u64
```

Proposes a new transaction for approval.

**Parameters:**
- `to` - Recipient address
- `value` - Amount of CHERT to send
- `data` - Contract call data (empty for simple transfers)
- `description` - Human-readable transaction description

**Returns:** Transaction ID

**Requirements:**
- Caller must be an owner
- Recipient address must be valid

**Events:**
- `TransactionSubmitted { tx_id, proposer, to, value }`

**Automatic Approval:** Proposer automatically approves the transaction

### Approve Transaction

```rust
fn approve_transaction(tx_id: u64)
```

Adds caller's signature to a pending transaction.

**Parameters:**
- `tx_id` - Transaction ID to approve

**Requirements:**
- Caller must be an owner
- Transaction must exist and be pending
- Caller must not have already approved
- Transaction must not be executed or cancelled

**Events:**
- `TransactionApproved { tx_id, approver }`

### Revoke Approval

```rust
fn revoke_approval(tx_id: u64)
```

Removes caller's signature from a pending transaction.

**Parameters:**
- `tx_id` - Transaction ID to revoke

**Requirements:**
- Caller must be an owner
- Transaction must be pending
- Caller must have previously approved
- Transaction must not be executed

**Events:**
- `ApprovalRevoked { tx_id, owner }`

### Execute Transaction

```rust
fn execute_transaction(tx_id: u64)
```

Executes a transaction that has reached threshold signatures.

**Parameters:**
- `tx_id` - Transaction ID to execute

**Requirements:**
- Transaction must exist and be pending
- Must have M or more approvals
- If time-locked, delay period must have passed
- Wallet must have sufficient balance (for value transfers)

**Events:**
- `TransactionExecuted { tx_id, executor }`
- On failure: `ExecutionFailed { tx_id, reason }`

### Cancel Transaction

```rust
fn cancel_transaction(tx_id: u64)
```

Cancels a pending transaction (requires M signatures).

**Parameters:**
- `tx_id` - Transaction ID to cancel

**Requirements:**
- Transaction must be pending
- Must have M approvals for cancellation
- Uses same signature mechanism as execution

**Events:**
- `TransactionCancelled { tx_id }`

### Add Owner

```rust
fn add_owner(owner: String)
```

Adds a new signer to the wallet (via multisig transaction).

**Parameters:**
- `owner` - New owner address to add

**Requirements:**
- Must be called via `execute_transaction` (requires M signatures)
- Owner must not already exist
- Total owners must not exceed maximum (50)

**Events:**
- `OwnerAdded { owner }`

### Remove Owner

```rust
fn remove_owner(owner: String)
```

Removes a signer from the wallet (via multisig transaction).

**Parameters:**
- `owner` - Owner address to remove

**Requirements:**
- Must be called via `execute_transaction` (requires M signatures)
- Owner must exist
- Remaining owners must be ≥ threshold
- Cannot remove last owner

**Events:**
- `OwnerRemoved { owner }`

### Change Threshold

```rust
fn change_threshold(new_threshold: u64)
```

Updates the required signature count (via multisig transaction).

**Parameters:**
- `new_threshold` - New threshold value (M)

**Requirements:**
- Must be called via `execute_transaction` (requires M signatures)
- Threshold > 0
- Threshold ≤ total owners

**Events:**
- `ThresholdChanged { old_threshold, new_threshold }`

### Set Time Lock

```rust
fn set_time_lock(tx_id: u64, delay_seconds: u64)
```

Adds an execution delay to a transaction for additional security.

**Parameters:**
- `tx_id` - Transaction ID
- `delay_seconds` - Seconds to wait after threshold reached

**Requirements:**
- Caller must be an owner
- Transaction must be pending
- Delay must be reasonable (< 30 days)

**Events:**
- `TimeLockSet { tx_id, unlock_time }`

## Query Functions

### Get Owners

```rust
fn get_owners() -> Vec<String>
```

Returns all current wallet owners.

**Returns:** Array of owner addresses

### Get Threshold

```rust
fn get_threshold() -> u64
```

Returns the current signature threshold.

**Returns:** Required signature count (M)

### Get Transaction

```rust
fn get_transaction(tx_id: u64) -> Transaction
```

Returns details of a specific transaction.

**Returns:** Transaction struct with all details

```rust
struct Transaction {
    to: String,
    value: u64,
    data: Vec<u8>,
    description: String,
    proposer: String,
    approvals: Vec<String>,
    executed: bool,
    cancelled: bool,
    timestamp: u64,
    time_lock: Option<u64>,
}
```

### Get Transaction Count

```rust
fn get_transaction_count() -> u64
```

Returns the total number of transactions (including executed/cancelled).

**Returns:** Transaction count

### Get Pending Transactions

```rust
fn get_pending_transactions() -> Vec<u64>
```

Returns IDs of all pending (not executed/cancelled) transactions.

**Returns:** Array of transaction IDs

### Has Approved

```rust
fn has_approved(tx_id: u64, owner: String) -> bool
```

Checks if an owner has approved a transaction.

**Returns:** True if approved, false otherwise

### Is Owner

```rust
fn is_owner(address: String) -> bool
```

Checks if an address is a wallet owner.

**Returns:** True if owner, false otherwise

### Get Approval Count

```rust
fn get_approval_count(tx_id: u64) -> u64
```

Returns the number of approvals for a transaction.

**Returns:** Approval count

### Can Execute

```rust
fn can_execute(tx_id: u64) -> bool
```

Checks if a transaction is ready to execute.

**Returns:** True if has threshold signatures and time lock expired

## Events

```rust
// Emitted when wallet is created
event WalletCreated {
    owners: Vec<String>,
    threshold: u64,
}

// Emitted when transaction is proposed
event TransactionSubmitted {
    tx_id: u64,
    proposer: String,
    to: String,
    value: u64,
    description: String,
}

// Emitted when transaction is approved
event TransactionApproved {
    tx_id: u64,
    approver: String,
    approval_count: u64,
}

// Emitted when approval is revoked
event ApprovalRevoked {
    tx_id: u64,
    owner: String,
}

// Emitted when transaction executes successfully
event TransactionExecuted {
    tx_id: u64,
    executor: String,
}

// Emitted when execution fails
event ExecutionFailed {
    tx_id: u64,
    reason: String,
}

// Emitted when transaction is cancelled
event TransactionCancelled {
    tx_id: u64,
}

// Emitted when owner is added
event OwnerAdded {
    owner: String,
}

// Emitted when owner is removed
event OwnerRemoved {
    owner: String,
}

// Emitted when threshold changes
event ThresholdChanged {
    old_threshold: u64,
    new_threshold: u64,
}

// Emitted when time lock is set
event TimeLockSet {
    tx_id: u64,
    unlock_time: u64,
}
```

## Storage Layout

```rust
// Owner management
Vector<String>: "owners"
Map<String, bool>: "is_owner"
u64: "threshold"

// Transaction storage
u64: "transaction_count"
Map<u64, Transaction>: "transactions"
Map<u64, Vec<String>>: "approvals"  // tx_id -> [approvers]
Map<(u64, String), bool>: "has_approved"  // (tx_id, owner) -> bool
Vector<u64>: "pending_txs"

// Time locks
Map<u64, u64>: "time_locks"  // tx_id -> unlock_timestamp

// Configuration
u64: "max_owners"  // 50
u64: "max_time_lock"  // 30 days
```

## Security Considerations

### Signature Verification
- Strict owner validation on all operations
- Prevent signature reuse across transactions
- Nonce tracking to prevent replay attacks

### Threshold Enforcement
- Cannot execute without M signatures
- Threshold changes require existing threshold
- Cannot set threshold > owner count

### Owner Management
- Owner changes require multisig approval
- Cannot remove owner if below threshold
- Prevent duplicate owner addresses

### Execution Safety
- Reentrancy protection on execution
- State changes before external calls
- Execution failure handling (doesn't revert entire transaction)
- Balance checks before transfers

### Time Lock Protection
- Optional delays for high-value transactions
- Prevents immediate execution after threshold
- Cancellation allowed during time lock period

### Integer Safety
- Overflow protection on all arithmetic
- Threshold bounds checking
- Transaction ID monotonic increase

## Example Usage

### Creating a 2-of-3 Multisig

```rust
let owners = vec![
    "chert_1alice...".to_string(),
    "chert_1bob...".to_string(),
    "chert_1charlie...".to_string(),
];

multisig.initialize(owners, 2);  // Requires 2 of 3 signatures
```

### Proposing a Transaction

```rust
// Alice proposes sending 1000 CHERT to Dave
let tx_id = multisig.submit_transaction(
    "chert_1dave...".to_string(),
    1000,  // 1000 CHERT
    vec![],  // No contract call data
    "Payment to contractor Dave".to_string()
);
// Alice's signature is automatically counted
```

### Approving a Transaction

```rust
// Bob approves the transaction
multisig.approve_transaction(tx_id);
// Now has 2/3 signatures, ready to execute
```

### Executing a Transaction

```rust
// Anyone can execute once threshold is reached
multisig.execute_transaction(tx_id);
// Transaction executes, Dave receives 1000 CHERT
```

### Adding a New Owner

```rust
// First, propose the owner addition as a transaction
let tx_id = multisig.submit_transaction(
    multisig_address,  // Call self
    0,  // No value transfer
    encode_call("add_owner", "chert_1eve..."),
    "Add Eve as fourth owner".to_string()
);

// Get 2 approvals (including proposer)
multisig.approve_transaction(tx_id);

// Execute
multisig.execute_transaction(tx_id);
// Now it's a 2-of-4 multisig
```

### Changing Threshold

```rust
// Increase to 3-of-4
let tx_id = multisig.submit_transaction(
    multisig_address,
    0,
    encode_call("change_threshold", 3),
    "Increase security to 3 signatures".to_string()
);

// Get current threshold (2) approvals
multisig.approve_transaction(tx_id);
multisig.execute_transaction(tx_id);
// Now requires 3 signatures
```

## Advanced Use Cases

### Treasury Management

```rust
// Deploy multisig for DAO treasury
let treasury = deploy_multisig(council_members, 5);  // 5-of-9 council

// Propose spending from treasury
let tx_id = treasury.submit_transaction(
    grant_recipient,
    50000,  // 50k CHERT grant
    vec![],
    "Q1 2026 Development Grant"
);

// Council members vote
for member in council_members[0..5] {
    treasury.approve_transaction(tx_id);
}

treasury.execute_transaction(tx_id);
```

### Time-Locked High-Value Transfer

```rust
// Propose large transfer
let tx_id = multisig.submit_transaction(
    recipient,
    1_000_000,  // 1M CHERT
    vec![],
    "Large partnership payment"
);

// Set 7-day time lock for review period
multisig.set_time_lock(tx_id, 7 * 24 * 60 * 60);

// Get approvals
multisig.approve_transaction(tx_id);
multisig.approve_transaction(tx_id);

// Must wait 7 days before execution
// ... 7 days later ...
multisig.execute_transaction(tx_id);
```

### Contract Upgrade Governance

```rust
// Propose contract upgrade
let tx_id = multisig.submit_transaction(
    protocol_contract,
    0,
    encode_call("upgrade", new_implementation),
    "Upgrade to v2.0"
);

// Security council reviews and approves
// Requires unanimous approval (5-of-5)
for member in security_council {
    multisig.approve_transaction(tx_id);
}

multisig.execute_transaction(tx_id);
```

## Differences from Gnosis Safe

- ✅ **Native Integration** - Built on Chert's consensus layer
- ✅ **Lower Gas Costs** - Optimized storage and execution
- ✅ **Post-Quantum Ready** - Compatible with Dilithium signatures
- ✅ **Built-in Time Locks** - No separate module needed
- ✅ **Simpler Architecture** - Focused on core multisig functionality

## Testing Checklist

- [ ] Initialize with various owner/threshold combinations
- [ ] Submit transactions (transfers and contract calls)
- [ ] Approve transactions by multiple owners
- [ ] Revoke approvals before execution
- [ ] Execute transactions at threshold
- [ ] Fail execution below threshold
- [ ] Add owners via multisig
- [ ] Remove owners via multisig
- [ ] Change threshold via multisig
- [ ] Cancel pending transactions
- [ ] Set and enforce time locks
- [ ] Handle execution failures gracefully
- [ ] Prevent duplicate approvals
- [ ] Prevent non-owner interactions
- [ ] Test reentrancy protection
- [ ] Test with maximum owners (50)

## License

MIT License - See LICENSE file for details

## References

- [Gnosis Safe](https://github.com/safe-global/safe-contracts)
- [MultiSigWallet](https://github.com/gnosis/MultiSigWallet)
- [EIP-1271: Standard Signature Validation](https://eips.ethereum.org/EIPS/eip-1271)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q1 2026
