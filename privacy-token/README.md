# Privacy Token (Confidential Transactions)

A privacy-preserving token with confidential balances and anonymous transfers on Chert Coin blockchain.

## Features

- ✅ **Confidential Balances** - Hidden account balances using cryptographic commitments
- ✅ **Anonymous Transfers** - Transfer amounts hidden from public view
- ✅ **Stealth Addresses** - One-time addresses for recipient privacy
- ✅ **Range Proofs** - Prove balances are positive without revealing amounts
- ✅ **Audit Keys** - Optional selective disclosure for compliance
- ✅ **Shielded Pool** - Private transaction pool
- ✅ **Public ↔ Private** - Bridge between transparent and private balances
- ✅ **Zero-Knowledge Proofs** - Cryptographic transaction validity

## Use Cases

- 🔒 **Private Payments** - Confidential business transactions
- 💼 **Payroll Privacy** - Hide employee compensation details
- 🏥 **Healthcare Payments** - HIPAA-compliant medical billing
- 💰 **Wealth Privacy** - Protect against targeted attacks
- 🤝 **B2B Transactions** - Confidential supplier payments
- 🎭 **Anonymous Donations** - Private charitable giving
- 📊 **Trading Privacy** - Hide trading strategies and positions

## Architecture

### Privacy Model

```
Transparent Pool (Public):
- Standard CRC-20 functionality
- Visible balances and transfers
- On-chain compliance

Shielded Pool (Private):
- Hidden balances (Pedersen commitments)
- Anonymous transfers (Bulletproofs)
- Stealth addresses for recipients
- Range proofs for validity

Bridge:
- Shield: Move tokens public → private
- Unshield: Move tokens private → public
```

### Cryptographic Primitives

```
Pedersen Commitments:
C = r*G + v*H
- r = random blinding factor
- v = value (balance or amount)
- G, H = elliptic curve generators

Range Proofs (Bulletproofs):
Prove: 0 ≤ v < 2^64
Without revealing v

Stealth Addresses:
Recipient generates one-time address
Sender can pay without revealing recipient
```

## API Reference

### Initialize

```rust
fn initialize(
    name: String,
    symbol: String,
    total_supply: u64,
    audit_key: Option<String>
)
```

Creates a privacy token with initial supply.

**Parameters:**
- `name` - Token name (e.g., "Private Chert")
- `symbol` - Token symbol (e.g., "pCHERT")
- `total_supply` - Initial supply (starts in deployer's transparent balance)
- `audit_key` - Optional auditor public key for regulatory compliance

**Requirements:**
- Total supply must be reasonable
- Audit key must be valid if provided

**Events:**
- `TokenCreated { name, symbol, total_supply, has_audit_key }`

### Shield (Public → Private)

```rust
fn shield(
    amount: u64,
    note_commitment: [u8; 32],
    range_proof: Vec<u8>
) -> [u8; 32]
```

Converts transparent tokens to private tokens.

**Parameters:**
- `amount` - Amount to shield (visible, deducted from transparent balance)
- `note_commitment` - Pedersen commitment to shielded note
- `range_proof` - Bulletproof proving 0 ≤ amount < 2^64

**Returns:** Note hash (identifier for private note)

**Requirements:**
- Caller must have sufficient transparent balance
- Range proof must be valid
- Note commitment must be well-formed

**Events:**
- `Shield { from: caller, amount, note_hash }`

**Privacy:** Amount is visible during shielding, but subsequent transfers are private

### Private Transfer

```rust
fn private_transfer(
    input_notes: Vec<[u8; 32]>,
    output_commitments: Vec<[u8; 32]>,
    range_proofs: Vec<Vec<u8>>,
    signature: Vec<u8>,
    public_change: i64
) -> Vec<[u8; 32]>
```

Performs a confidential transfer between shielded accounts.

**Parameters:**
- `input_notes` - Nullifiers for consumed notes (prevent double-spending)
- `output_commitments` - Pedersen commitments for new notes
- `range_proofs` - Bulletproofs for each output
- `signature` - Signature over transaction
- `public_change` - Change to transparent balance (can be 0)

**Returns:** Output note hashes

**Requirements:**
- Input notes must exist and not be spent
- Sum of inputs = Sum of outputs (in commitment space)
- Range proofs valid for all outputs
- Signature valid
- Nullifiers not previously used

**Events:**
- `PrivateTransfer { nullifiers: input_notes, commitments: output_commitments }`

**Privacy:** Amounts, sender, and recipient are hidden

### Unshield (Private → Public)

```rust
fn unshield(
    input_notes: Vec<[u8; 32]>,
    amount: u64,
    recipient: String,
    change_commitment: Option<[u8; 32]>,
    range_proof: Vec<u8>,
    signature: Vec<u8>
)
```

Converts private tokens back to transparent tokens.

**Parameters:**
- `input_notes` - Nullifiers for consumed shielded notes
- `amount` - Amount to unshield (becomes visible)
- `recipient` - Recipient address for transparent tokens
- `change_commitment` - Commitment for remaining private balance
- `range_proof` - Proof for change commitment
- `signature` - Authorization signature

**Requirements:**
- Input notes must exist and be unspent
- Sum of inputs ≥ amount + change
- Range proof valid if change exists
- Signature valid

**Events:**
- `Unshield { to: recipient, amount, nullifiers }`

**Privacy:** Only unshielded amount is revealed, source remains private

### Generate Stealth Address

```rust
fn generate_stealth_address(
    recipient_public_key: [u8; 32],
    random_secret: [u8; 32]
) -> ([u8; 32], [u8; 32])
```

Generates a one-time stealth address for recipient.

**Parameters:**
- `recipient_public_key` - Recipient's public viewing key
- `random_secret` - Sender's ephemeral secret

**Returns:** `(stealth_address, ephemeral_public_key)`

**Off-Chain:** Sender publishes ephemeral_public_key with transaction
**Recipient:** Scans blockchain using viewing key to detect payments

### Audit Balance

```rust
fn audit_balance(
    commitment: [u8; 32],
    viewing_key: [u8; 32],
    audit_signature: Vec<u8>
) -> u64
```

Allows authorized auditor to view specific balance.

**Parameters:**
- `commitment` - Balance commitment to audit
- `viewing_key` - Decryption key for commitment
- `audit_signature` - Auditor's authorization signature

**Returns:** Decrypted balance amount

**Requirements:**
- Audit key must be configured
- Audit signature must be valid
- Viewing key must decrypt commitment correctly

**Use Case:** Regulatory compliance, tax reporting

## Query Functions

### Get Transparent Balance

```rust
fn balance_of(account: String) -> u64
```

Returns transparent (public) balance.

**Returns:** Token amount

### Get Shielded Pool Size

```rust
fn shielded_pool_size() -> u64
```

Returns total value in shielded pool.

**Returns:** Total shielded tokens (visible as aggregate)

**Privacy:** Individual balances remain hidden

### Is Note Spent

```rust
fn is_note_spent(nullifier: [u8; 32]) -> bool
```

Checks if a note has been consumed.

**Returns:** True if spent

### Verify Range Proof

```rust
fn verify_range_proof(
    commitment: [u8; 32],
    proof: Vec<u8>
) -> bool
```

Verifies a Bulletproof range proof.

**Returns:** True if proof is valid

### Get Audit Key

```rust
fn get_audit_key() -> Option<[u8; 32]>
```

Returns the configured audit public key.

**Returns:** Audit key if configured

## Events

```rust
// Emitted when token is created
event TokenCreated {
    name: String,
    symbol: String,
    total_supply: u64,
    has_audit_key: bool,
}

// Emitted when tokens are shielded
event Shield {
    from: String,
    amount: u64,
    note_hash: [u8; 32],
}

// Emitted when tokens are unshielded
event Unshield {
    to: String,
    amount: u64,
    nullifiers: Vec<[u8; 32]>,
}

// Emitted for private transfers
event PrivateTransfer {
    nullifiers: Vec<[u8; 32]>,
    commitments: Vec<[u8; 32]>,
    ephemeral_key: [u8; 32],  // For stealth address scanning
}

// Emitted when balance is audited
event BalanceAudited {
    auditor: String,
    commitment: [u8; 32],
    timestamp: u64,
}

// Standard CRC-20 events for transparent transfers
event Transfer {
    from: String,
    to: String,
    amount: u64,
}
```

## Storage Layout

```rust
// Standard CRC-20 (transparent)
Map<String, u64>: "balances"
Map<(String, String), u64>: "allowances"
u64: "total_supply"
String: "name"
String: "symbol"

// Shielded pool
Map<[u8; 32], bool>: "note_commitments"  // commitment -> exists
Map<[u8; 32], bool>: "nullifiers"        // nullifier -> spent
u64: "shielded_pool_total"

// Merkle tree for note set
Vector<[u8; 32]>: "note_tree"  // Sparse Merkle tree
u64: "tree_depth"

// Audit configuration
Option<[u8; 32]>: "audit_public_key"

// Cryptographic parameters
[u8; 32]: "generator_G"
[u8; 32]: "generator_H"
```

## Security Considerations

### Cryptographic Security
- **Pedersen Commitments**: Computationally hiding and binding
- **Bulletproofs**: Efficient range proofs (64-2048 bit ranges)
- **Schnorr Signatures**: Provably secure authorization
- **Elliptic Curve**: secp256k1 or Ristretto255

### Privacy Guarantees
- **Amount Privacy**: Values hidden in commitments
- **Sender Privacy**: Stealth addresses hide sender
- **Recipient Privacy**: One-time addresses hide recipient
- **Transaction Graph**: Mixing breaks on-chain analysis

### Compliance Features
- **Audit Keys**: Optional selective disclosure
- **Transparent Bridge**: Maintain regulatory interfaces
- **Note Metadata**: Optional encrypted memos

### Anti-Double-Spend
- **Nullifiers**: Unique identifiers prevent note reuse
- **Nullifier Set**: Track all spent notes
- **Merkle Tree**: Prove note existence without revealing it

### Economic Security
- **No Inflation**: Range proofs ensure no negative balances
- **Conservation**: Input sum = Output sum enforced
- **No Hidden Mint**: All supply starts transparent

## Mathematical Details

### Pedersen Commitment

```
Commit(value, blinding):
  C = value * H + blinding * G

Properties:
- Hiding: Cannot determine value from C
- Binding: Cannot find two (value, blinding) pairs with same C
- Homomorphic: C1 + C2 = Commit(v1 + v2, r1 + r2)
```

### Range Proof (Bulletproof)

```
Prove: 0 ≤ value < 2^64
Proof size: ~700 bytes (64-bit range)
Verification: ~2ms

Aggregation: Prove multiple ranges in one proof
N ranges: ~700 + 100*N bytes
```

### Stealth Address

```
Sender:
- Generate random r
- Compute R = r*G (ephemeral public key)
- Compute shared secret: s = r*B (B = recipient public key)
- Stealth address: P = H(s)*G + B

Recipient:
- Sees R on blockchain
- Compute shared secret: s = b*R (b = recipient private key)
- Check if P = H(s)*G + B
- Derive private key: p = H(s) + b
```

## Example Usage

### Creating Privacy Token

```rust
let privacy_token = deploy_privacy_token();

privacy_token.initialize(
    "Private Chert".to_string(),
    "pCHERT".to_string(),
    1_000_000,  // 1M total supply
    Some(regulatory_auditor_key)  // Enable auditing
);
```

### Shielding Tokens

```rust
// Alice shields 1000 tokens
let amount = 1000;
let blinding = random_scalar();
let commitment = pedersen_commit(amount, blinding);
let range_proof = generate_bulletproof(amount, blinding);

let note_hash = privacy_token.shield(
    amount,
    commitment,
    range_proof
);

// Alice stores (note_hash, amount, blinding) privately
// Her transparent balance decreases by 1000
// Her shielded balance increases by 1000 (only she knows this)
```

### Private Transfer

```rust
// Alice sends 300 tokens to Bob privately

// Alice's inputs: 1000 token note
let input_nullifier = compute_nullifier(alice_note, alice_key);

// Outputs: 300 to Bob, 700 change to Alice
let bob_amount = 300;
let bob_blinding = random_scalar();
let bob_commitment = pedersen_commit(bob_amount, bob_blinding);

let change_amount = 700;
let change_blinding = random_scalar();
let change_commitment = pedersen_commit(change_amount, change_blinding);

// Generate stealth address for Bob
let (bob_stealth, ephemeral_key) = generate_stealth_address(
    bob_viewing_key,
    random_secret()
);

// Generate range proofs
let bob_proof = generate_bulletproof(bob_amount, bob_blinding);
let change_proof = generate_bulletproof(change_amount, change_blinding);

// Sign transaction
let signature = sign_transaction(alice_key, tx_data);

// Submit private transfer
privacy_token.private_transfer(
    vec![input_nullifier],
    vec![bob_commitment, change_commitment],
    vec![bob_proof, change_proof],
    signature,
    0  // No public change
);

// Bob scans blockchain with his viewing key
// Detects payment to his stealth address
// Derives private key to spend the note
```

### Unshielding Tokens

```rust
// Bob unshields 300 tokens to his public address

let input_nullifier = compute_nullifier(bob_note, bob_key);
let signature = sign(bob_key, unshield_data);

privacy_token.unshield(
    vec![input_nullifier],
    300,  // Amount becomes public
    bob_public_address,
    None,  // No shielded change
    vec![],  // No range proof needed
    signature
);

// Bob's transparent balance increases by 300
// Amount 300 is now visible on-chain
```

### Regulatory Audit

```rust
// Auditor reviews Alice's shielded balance for compliance

let viewing_key = alice_provides_viewing_key();
let audit_sig = auditor_signs_authorization();

let balance = privacy_token.audit_balance(
    alice_commitment,
    viewing_key,
    audit_sig
);

// Auditor learns Alice's balance: 700 tokens
// This action is logged on-chain
```

## Integration Examples

### Private DEX

```rust
// Trade privacy tokens without revealing amounts
fn private_swap(
    input_notes: Vec<[u8; 32]>,
    output_commitments: Vec<[u8; 32]>,
    proofs: Vec<Vec<u8>>
) {
    // Verify input ownership
    // Verify output commitments match swap ratio
    // Execute swap preserving privacy
}
```

### Privacy-Preserving DeFi

```rust
// Deposit collateral privately
fn private_collateral_deposit(
    amount_commitment: [u8; 32],
    range_proof: Vec<u8>
) {
    // Verify sufficient collateral via ZK proof
    // Don't reveal exact amount
    // Allow borrowing based on proof
}
```

## Differences from Monero/Zcash

- ✅ **Smart Contract**: Built as contract, not protocol layer
- ✅ **Optional Privacy**: Can choose public or private
- ✅ **Transparent Bridge**: Easy on/off ramps
- ✅ **Audit Support**: Built-in compliance features
- ✅ **Lighter Weight**: Bulletproofs vs. zk-SNARKs

## Testing Checklist

- [ ] Initialize privacy token
- [ ] Shield tokens (public → private)
- [ ] Private transfer with single input/output
- [ ] Private transfer with multiple inputs/outputs
- [ ] Unshield tokens (private → public)
- [ ] Generate and detect stealth addresses
- [ ] Verify range proofs
- [ ] Prevent double-spending (nullifier check)
- [ ] Audit shielded balance with viewing key
- [ ] Test with maximum transaction size
- [ ] Verify commitment homomorphism
- [ ] Test transparent CRC-20 functionality
- [ ] Aggregate bulletproofs for efficiency

## License

MIT License - See LICENSE file for details

## References

- [Bulletproofs Paper](https://eprint.iacr.org/2017/1066.pdf)
- [Monero Stealth Addresses](https://www.getmonero.org/resources/moneropedia/stealthaddress.html)
- [Zcash Protocol](https://zips.z.cash/protocol/protocol.pdf)
- [Pedersen Commitments](https://crypto.stanford.edu/~dabo/pubs/papers/crypto91.pdf)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q2 2026

**Note:** This is a complex cryptographic system requiring thorough security audits before production use.
