# Cross-Shard Bridge

A secure cross-shard communication and asset transfer contract for Chert Coin's sharded blockchain architecture.

## Features

- ✅ **Asset Bridging** - Transfer tokens between shards
- ✅ **Message Passing** - Cross-shard contract calls
- ✅ **Atomic Swaps** - Exchange assets across shards atomically
- ✅ **State Verification** - Cryptographic proof of shard state
- ✅ **Fraud Proofs** - Challenge invalid cross-shard transfers
- ✅ **Optimistic Bridging** - Fast transfers with security guarantees
- ✅ **Multi-Hop Routing** - Transfers through intermediate shards
- ✅ **Gas Abstraction** - Pay fees on source shard

## Use Cases

- 💱 **Cross-Shard DEX** - Trade assets from different shards
- 🎮 **Gaming Assets** - Move items between game shards
- 💰 **Shard Balancing** - Optimize liquidity distribution
- 🔗 **Composability** - Call contracts on other shards
- 🌐 **DApp Scalability** - Distribute dApp across shards
- 📊 **Data Aggregation** - Query state from multiple shards

## Architecture

### Chert's Sharding Model

```
Beacon Chain (Shard 0):
- Coordinates cross-shard communication
- Maintains shard state roots
- Validates cross-shard receipts

Application Shards (1-N):
- Process transactions independently
- Maintain local state
- Generate cross-shard messages

Bridge Contracts:
- One per shard
- Lock/unlock assets
- Verify cross-shard proofs
```

### Transfer Flow

```
Source Shard:
1. Lock assets in bridge contract
2. Generate cross-shard receipt
3. Submit to beacon chain

Beacon Chain:
4. Verify receipt validity
5. Update shard state roots
6. Create cross-shard message

Target Shard:
7. Prove message inclusion in beacon
8. Verify Merkle proof
9. Unlock/mint assets
```

## API Reference

### Initialize

```rust
fn initialize(
    shard_id: u64,
    beacon_address: String,
    validator_set: Vec<String>
)
```

Initializes the bridge contract for a specific shard.

**Parameters:**
- `shard_id` - This shard's unique identifier
- `beacon_address` - Beacon chain bridge contract address
- `validator_set` - Initial set of shard validators

**Requirements:**
- Shard ID must be valid (1-255)
- Beacon address must be valid
- Minimum validator count (e.g., 10)

**Events:**
- `BridgeInitialized { shard_id, beacon_address }`

### Lock and Bridge

```rust
fn lock_and_bridge(
    token: String,
    amount: u64,
    target_shard: u64,
    recipient: String,
    timeout: u64
) -> [u8; 32]
```

Locks tokens and initiates cross-shard transfer.

**Parameters:**
- `token` - Token contract address to bridge
- `amount` - Amount to transfer
- `target_shard` - Destination shard ID
- `recipient` - Recipient address on target shard
- `timeout` - Block number for refund if transfer fails

**Returns:** Transfer ID (hash of transfer parameters)

**Requirements:**
- Caller must have sufficient balance
- Token must be approved for bridge
- Target shard must be active
- Amount must be > 0

**Events:**
- `AssetLocked { transfer_id, token, amount, target_shard, recipient }`

**Process:**
1. Locks tokens in escrow
2. Generates cross-shard receipt
3. Submits to beacon chain

### Claim Bridged Assets

```rust
fn claim_bridged_assets(
    transfer_id: [u8; 32],
    source_shard: u64,
    token: String,
    amount: u64,
    recipient: String,
    merkle_proof: Vec<[u8; 32]>,
    beacon_root: [u8; 32]
)
```

Claims assets on target shard using Merkle proof.

**Parameters:**
- `transfer_id` - Unique transfer identifier
- `source_shard` - Originating shard ID
- `token` - Token being bridged
- `amount` - Amount to claim
- `recipient` - Recipient address (must be caller)
- `merkle_proof` - Proof of inclusion in beacon state
- `beacon_root` - Beacon chain state root

**Requirements:**
- Caller must be recipient
- Proof must be valid against beacon root
- Transfer must not already be claimed
- Beacon root must be finalized

**Events:**
- `AssetsClaimed { transfer_id, recipient, amount }`

**Process:**
1. Verifies Merkle proof against beacon
2. Mints/unlocks tokens on target shard
3. Marks transfer as completed

### Send Cross-Shard Message

```rust
fn send_message(
    target_shard: u64,
    target_contract: String,
    calldata: Vec<u8>,
    gas_limit: u64
) -> [u8; 32]
```

Sends a message to contract on another shard.

**Parameters:**
- `target_shard` - Destination shard ID
- `target_contract` - Contract address to call
- `calldata` - Function call data
- `gas_limit` - Maximum gas for execution on target

**Returns:** Message ID

**Requirements:**
- Target shard must be active
- Caller must pay cross-shard fee
- Gas limit must be reasonable

**Events:**
- `MessageSent { message_id, target_shard, target_contract }`

### Execute Cross-Shard Message

```rust
fn execute_message(
    message_id: [u8; 32],
    source_shard: u64,
    source_contract: String,
    calldata: Vec<u8>,
    gas_limit: u64,
    merkle_proof: Vec<[u8; 32]>,
    beacon_root: [u8; 32]
)
```

Executes a message from another shard.

**Parameters:**
- `message_id` - Unique message identifier
- `source_shard` - Originating shard ID
- `source_contract` - Sender contract address
- `calldata` - Function call data
- `gas_limit` - Gas limit for execution
- `merkle_proof` - Proof of message in beacon
- `beacon_root` - Beacon state root

**Requirements:**
- Proof must be valid
- Message not already executed
- Gas limit not exceeded

**Events:**
- `MessageExecuted { message_id, success, return_data }`

### Challenge Transfer

```rust
fn challenge_transfer(
    transfer_id: [u8; 32],
    fraud_proof: Vec<u8>
)
```

Challenges an invalid cross-shard transfer.

**Parameters:**
- `transfer_id` - Transfer to challenge
- `fraud_proof` - Proof of fraud (invalid signature, double-spend, etc.)

**Requirements:**
- Transfer must be in challenge period
- Fraud proof must be valid
- Challenger must stake bond

**Events:**
- `TransferChallenged { transfer_id, challenger }`

**Effects:**
- If valid: Reverses transfer, slashes validators, rewards challenger
- If invalid: Slashes challenger stake

### Refund Timeout

```rust
fn refund_timeout(transfer_id: [u8; 32])
```

Refunds locked assets if transfer times out.

**Parameters:**
- `transfer_id` - Transfer to refund

**Requirements:**
- Transfer must exist
- Timeout block must be reached
- Transfer must not be completed

**Events:**
- `TransferRefunded { transfer_id, amount }`

**Process:**
1. Unlocks original assets
2. Returns to sender
3. Marks transfer as cancelled

## Query Functions

### Get Transfer Status

```rust
fn get_transfer_status(transfer_id: [u8; 32]) -> TransferStatus
```

Returns the current status of a transfer.

**Returns:** Transfer status

```rust
enum TransferStatus {
    Pending,      // Locked, waiting for claim
    Completed,    // Successfully claimed
    Challenged,   // Under dispute
    Refunded,     // Timed out and refunded
}
```

### Get Shard State Root

```rust
fn get_shard_state_root(shard_id: u64, block_number: u64) -> [u8; 32]
```

Returns the state root of a shard at specific block.

**Returns:** State root hash

### Verify Cross-Shard Proof

```rust
fn verify_cross_shard_proof(
    message_hash: [u8; 32],
    merkle_proof: Vec<[u8; 32]>,
    beacon_root: [u8; 32]
) -> bool
```

Verifies a Merkle proof against beacon root.

**Returns:** True if proof is valid

### Is Shard Active

```rust
fn is_shard_active(shard_id: u64) -> bool
```

Checks if a shard is currently active.

**Returns:** True if active

### Get Bridge Balance

```rust
fn get_bridge_balance(token: String) -> u64
```

Returns total locked balance for a token.

**Returns:** Token amount

### Estimate Cross-Shard Fee

```rust
fn estimate_cross_shard_fee(
    target_shard: u64,
    gas_limit: u64
) -> u64
```

Estimates fee for cross-shard operation.

**Returns:** Fee in CHERT tokens

## Events

```rust
// Emitted when bridge is initialized
event BridgeInitialized {
    shard_id: u64,
    beacon_address: String,
}

// Emitted when assets are locked
event AssetLocked {
    transfer_id: [u8; 32],
    sender: String,
    token: String,
    amount: u64,
    target_shard: u64,
    recipient: String,
    timeout: u64,
}

// Emitted when assets are claimed
event AssetsClaimed {
    transfer_id: [u8; 32],
    recipient: String,
    token: String,
    amount: u64,
}

// Emitted when transfer is refunded
event TransferRefunded {
    transfer_id: [u8; 32],
    sender: String,
    amount: u64,
}

// Emitted when message is sent
event MessageSent {
    message_id: [u8; 32],
    sender: String,
    target_shard: u64,
    target_contract: String,
    calldata: Vec<u8>,
}

// Emitted when message is executed
event MessageExecuted {
    message_id: [u8; 32],
    success: bool,
    return_data: Vec<u8>,
}

// Emitted when transfer is challenged
event TransferChallenged {
    transfer_id: [u8; 32],
    challenger: String,
    fraud_proof: Vec<u8>,
}

// Emitted when challenge is resolved
event ChallengeResolved {
    transfer_id: [u8; 32],
    valid: bool,
    slashed_party: String,
}
```

## Storage Layout

```rust
// Bridge configuration
u64: "shard_id"
String: "beacon_address"
Vector<String>: "validators"

// Transfer tracking
Map<[u8; 32], Transfer>: "transfers"
Map<[u8; 32], bool>: "completed_transfers"
Map<[u8; 32], bool>: "claimed_transfers"

// Token balances
Map<String, u64>: "locked_balances"  // token -> total_locked

// Cross-shard messages
Map<[u8; 32], Message>: "messages"
Map<[u8; 32], bool>: "executed_messages"

// State roots (from beacon)
Map<(u64, u64), [u8; 32]>: "shard_roots"  // (shard_id, block) -> root

// Challenge system
Map<[u8; 32], Challenge>: "challenges"
u64: "challenge_period"  // 1 day
u64: "challenge_bond"    // 1000 CHERT

struct Transfer {
    sender: String,
    token: String,
    amount: u64,
    source_shard: u64,
    target_shard: u64,
    recipient: String,
    status: TransferStatus,
    timeout_block: u64,
    timestamp: u64,
}

struct Message {
    sender: String,
    source_shard: u64,
    target_shard: u64,
    target_contract: String,
    calldata: Vec<u8>,
    gas_limit: u64,
    executed: bool,
}
```

## Security Considerations

### Cryptographic Security
- **Merkle Proofs**: Prove inclusion in beacon state
- **State Roots**: Cryptographic commitment to shard state
- **Validator Signatures**: Multi-signature validation
- **Hash Security**: BLAKE3 for all hashing

### Economic Security
- **Challenge Bonds**: Stake required to challenge
- **Validator Stakes**: Slashing for fraud
- **Timeout Protection**: Refunds for stuck transfers
- **Fee Market**: Dynamic pricing for congestion

### Fraud Prevention
- **Optimistic Bridging**: Assume valid, allow challenges
- **Fraud Proofs**: Prove invalid state transitions
- **Challenge Period**: Time window for disputes
- **Validator Slashing**: Economic penalty for fraud

### Availability
- **Timeout Refunds**: Recover from unavailable shards
- **Beacon Fallback**: Beacon chain provides finality
- **Multi-Hop Routing**: Route around offline shards

### Reentrancy Protection
- **Checks-Effects-Interactions**: State before external calls
- **Claim Once**: Prevent double-claiming
- **Message Replay**: Prevent message re-execution

## Example Usage

### Bridging Tokens Between Shards

```rust
// Alice on Shard 1 sends 1000 USDC to Bob on Shard 3

// 1. Lock tokens on Shard 1
let transfer_id = bridge_shard1.lock_and_bridge(
    usdc_token,
    1000,
    3,  // Target shard
    bob_address,
    current_block + 1000  // Timeout after 1000 blocks
);

// 2. Wait for beacon chain to process (automatic)
// Bridge generates receipt and submits to beacon

// 3. Bob claims on Shard 3 using Merkle proof
let (merkle_proof, beacon_root) = fetch_cross_shard_proof(transfer_id);

bridge_shard3.claim_bridged_assets(
    transfer_id,
    1,  // Source shard
    usdc_token,
    1000,
    bob_address,
    merkle_proof,
    beacon_root
);

// Bob receives 1000 USDC on Shard 3
```

### Cross-Shard Contract Call

```rust
// DEX on Shard 2 queries price oracle on Shard 4

// 1. Send message from Shard 2
let message_id = bridge_shard2.send_message(
    4,  // Target shard
    price_oracle_address,
    encode_call("get_price", "BTC/USD"),
    100_000  // Gas limit
);

// 2. Relayer picks up message and submits to Shard 4
let (proof, root) = fetch_message_proof(message_id);

bridge_shard4.execute_message(
    message_id,
    2,  // Source shard
    dex_address,
    encode_call("get_price", "BTC/USD"),
    100_000,
    proof,
    root
);

// 3. Oracle executes and optionally sends response back
```

### Atomic Cross-Shard Swap

```rust
// Alice (Shard 1) wants to swap 100 TOKEN_A for Bob's 200 TOKEN_B (Shard 2)

// 1. Both lock assets with hash time lock
let hash_lock = hash(secret);

alice_locks_token_a(100, hash_lock, timeout_1, bob_address);
bob_locks_token_b(200, hash_lock, timeout_2, alice_address);

// 2. Alice claims Bob's tokens by revealing secret
alice_claims_token_b(secret);  // On Shard 2

// 3. Bob sees secret, claims Alice's tokens
bob_claims_token_a(secret);  // On Shard 1

// Atomic: Either both succeed or both timeout and refund
```

### Multi-Hop Routing

```rust
// Transfer from Shard 1 → Shard 3 via Shard 2 (Shard 3 temporarily offline)

// Hop 1: Shard 1 → Shard 2
let hop1_id = bridge_shard1.lock_and_bridge(
    token, 1000, 2, intermediate_address, timeout
);

// Hop 2: Shard 2 → Shard 3 (when Shard 3 comes back online)
let hop2_id = bridge_shard2.lock_and_bridge(
    token, 1000, 3, final_recipient, timeout
);

// Final claim on Shard 3
bridge_shard3.claim_bridged_assets(...);
```

## Integration Examples

### Cross-Shard DEX

```rust
// Trade assets from different shards
fn cross_shard_trade(
    sell_token_shard: u64,
    sell_amount: u64,
    buy_token_shard: u64,
    min_buy_amount: u64
) {
    // 1. Bridge sell token to DEX shard
    let sell_transfer_id = bridge.lock_and_bridge(
        sell_token, sell_amount, dex_shard, dex_address, timeout
    );
    
    // 2. Execute trade on DEX shard
    // 3. Bridge purchased tokens to buyer's shard
    let buy_transfer_id = bridge.lock_and_bridge(
        buy_token, buy_amount, buyer_shard, buyer_address, timeout
    );
}
```

### Cross-Shard Lending

```rust
// Collateral on Shard 1, borrow from pool on Shard 2
fn cross_shard_borrow(
    collateral_amount: u64,
    borrow_amount: u64
) {
    // Lock collateral with proof
    let collateral_id = bridge.lock_and_bridge(
        collateral_token, collateral_amount, lending_shard, lending_pool, timeout
    );
    
    // Prove collateral locked, borrow on lending shard
    lending_pool.borrow_with_cross_shard_collateral(
        collateral_id, borrow_amount
    );
}
```

## Differences from Ethereum L2 Bridges

- ✅ **Native Sharding**: Built into protocol, not separate chains
- ✅ **Faster Finality**: Beacon provides quick finality
- ✅ **Lower Fees**: Internal routing, no external validators
- ✅ **Composability**: Call contracts across shards natively
- ✅ **Unified Security**: Same validator set secures all shards

## Testing Checklist

- [ ] Initialize bridge on multiple shards
- [ ] Lock and bridge tokens to another shard
- [ ] Claim bridged assets with valid proof
- [ ] Reject invalid Merkle proofs
- [ ] Send cross-shard messages
- [ ] Execute cross-shard messages
- [ ] Challenge fraudulent transfers
- [ ] Refund timed-out transfers
- [ ] Test multi-hop routing
- [ ] Test atomic swaps across shards
- [ ] Verify beacon state root updates
- [ ] Test with maximum transfer queue
- [ ] Simulate offline shard recovery
- [ ] Test reentrancy protection

## License

MIT License - See LICENSE file for details

## References

- [Ethereum Sharding](https://ethereum.org/en/upgrades/shard-chains/)
- [Polkadot XCMP](https://wiki.polkadot.network/docs/learn-crosschain)
- [Near Rainbow Bridge](https://near.org/bridge/)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q2 2026

**Note:** Requires coordination with Silica's beacon chain implementation.
