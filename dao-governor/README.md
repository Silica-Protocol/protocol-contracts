# DAO Governor

A comprehensive on-chain governance system for decentralized autonomous organizations on Chert Coin blockchain.

## Features

- ✅ **Token-Based Voting** - Governance power proportional to token holdings
- ✅ **Proposal System** - Submit, vote, and execute governance proposals
- ✅ **Quorum Requirements** - Minimum participation thresholds
- ✅ **Time Delays** - Voting periods and execution timelock
- ✅ **Delegation** - Transfer voting power to representatives
- ✅ **Veto Power** - Optional guardian veto for security
- ✅ **Snapshot Voting** - Prevent vote buying with historical balances
- ✅ **Quadratic Voting** - Optional: reduce whale influence
- ✅ **Treasury Control** - Govern protocol funds
- ✅ **Parameter Updates** - Change system configuration on-chain

## Use Cases

- 🏛️ **Protocol Governance** - Upgrade smart contracts and parameters
- 💰 **Treasury Management** - Allocate community funds
- 🎯 **Grant Programs** - Fund development and ecosystem growth
- ⚖️ **Dispute Resolution** - Community-driven arbitration
- 🔧 **Feature Activation** - Enable new protocol functionality
- 📜 **Constitutional Changes** - Modify governance rules
- 🤝 **Partnership Decisions** - Strategic alliance voting

## Architecture

### Governance Flow

```
1. Propose → Token holder submits proposal
2. Delay → Voting delay before voting starts
3. Vote → Active voting period
4. Queue → Successful proposal queued in timelock
5. Delay → Timelock delay before execution
6. Execute → Proposal executes on-chain

Timeline:
[Proposal] --2 days--> [Voting Start] --7 days--> [Voting End] --2 days--> [Execution]
           Delay                      Active            Timelock
```

### Voting Power

```
Standard Voting:
- 1 token = 1 vote
- Simple majority wins (>50%)
- Quorum required (e.g., 4% of supply)

Quadratic Voting (Optional):
- voting_power = sqrt(tokens)
- Reduces whale influence
- More democratic but complex
```

## API Reference

### Initialize

```rust
fn initialize(
    voting_token: String,
    timelock: String,
    voting_delay: u64,
    voting_period: u64,
    proposal_threshold: u64,
    quorum_percentage: u64
)
```

Initializes the DAO governor contract.

**Parameters:**
- `voting_token` - Governance token contract address
- `timelock` - Timelock controller address (executes proposals)
- `voting_delay` - Blocks before voting starts (e.g., 13,140 = ~2 days)
- `voting_period` - Blocks for voting (e.g., 45,990 = ~7 days)
- `proposal_threshold` - Minimum tokens to propose (e.g., 100,000)
- `quorum_percentage` - Minimum participation (e.g., 400 = 4%)

**Requirements:**
- All addresses must be valid
- Thresholds must be reasonable
- Quorum must be ≤ 100%

**Events:**
- `GovernorInitialized { voting_token, timelock, params }`

### Propose

```rust
fn propose(
    targets: Vec<String>,
    values: Vec<u64>,
    calldatas: Vec<Vec<u8>>,
    description: String
) -> u64
```

Creates a new governance proposal.

**Parameters:**
- `targets` - Contract addresses to call
- `values` - CHERT amounts to send with calls
- `calldatas` - Function call data for each target
- `description` - Detailed proposal description (Markdown supported)

**Returns:** Proposal ID

**Requirements:**
- Caller must have ≥ proposal_threshold tokens
- All parameter arrays must be same length
- Description must not be empty
- Cannot have identical active proposal

**Events:**
- `ProposalCreated { proposal_id, proposer, targets, description, vote_start, vote_end }`

**Examples:**
- Treasury spending: `propose([treasury], [1000], [transfer(recipient)], "Q1 Grant")`
- Parameter change: `propose([protocol], [0], [setFee(200)], "Reduce fee to 2%")`
- Contract upgrade: `propose([proxy], [0], [upgradeTo(impl_v2)], "Upgrade to v2")`

### Cast Vote

```rust
fn cast_vote(proposal_id: u64, support: VoteType) -> u64
```

Casts a vote on an active proposal.

**Parameters:**
- `proposal_id` - Proposal to vote on
- `support` - Vote type (For, Against, Abstain)

**Returns:** Voting power used

```rust
enum VoteType {
    Against = 0,
    For = 1,
    Abstain = 2,  // Counts toward quorum but not for/against
}
```

**Requirements:**
- Proposal must be active (in voting period)
- Caller must have voting power (at snapshot)
- Cannot vote twice

**Events:**
- `VoteCast { voter, proposal_id, support, weight, reason }`

### Cast Vote With Reason

```rust
fn cast_vote_with_reason(
    proposal_id: u64,
    support: VoteType,
    reason: String
) -> u64
```

Casts a vote with explanation.

**Parameters:**
- Same as `cast_vote` plus:
- `reason` - Explanation for vote decision

**Events:**
- `VoteCast { voter, proposal_id, support, weight, reason }`

**Use Case:** Transparent decision-making, delegate accountability

### Delegate

```rust
fn delegate(delegatee: String)
```

Delegates voting power to another address.

**Parameters:**
- `delegatee` - Address to delegate to (or self to reclaim)

**Requirements:**
- Delegatee must be valid address
- Cannot delegate to zero address

**Events:**
- `DelegateChanged { delegator, from_delegate, to_delegate }`
- `DelegateVotesChanged { delegate, previous_balance, new_balance }`

**Note:** Delegation is recursive (A→B→C means C votes with A's power)

### Queue Proposal

```rust
fn queue(proposal_id: u64)
```

Queues a successful proposal in the timelock for execution.

**Parameters:**
- `proposal_id` - Proposal to queue

**Requirements:**
- Proposal must have succeeded (quorum + majority)
- Voting period must be ended
- Proposal must not already be queued/executed

**Events:**
- `ProposalQueued { proposal_id, eta }`

**Effect:** Schedules proposal in timelock with delay

### Execute Proposal

```rust
fn execute(proposal_id: u64)
```

Executes a queued proposal after timelock delay.

**Parameters:**
- `proposal_id` - Proposal to execute

**Requirements:**
- Proposal must be queued
- Timelock delay must have passed
- Proposal must not be expired (grace period)

**Events:**
- `ProposalExecuted { proposal_id }`

**Effect:** Calls all target contracts with specified parameters

### Cancel Proposal

```rust
fn cancel(proposal_id: u64)
```

Cancels a proposal before execution.

**Parameters:**
- `proposal_id` - Proposal to cancel

**Requirements:**
- Caller must be proposer OR
- Proposer's tokens below threshold (lost support)
- Proposal must not be executed

**Events:**
- `ProposalCancelled { proposal_id }`

### Veto Proposal

```rust
fn veto(proposal_id: u64)
```

Emergency veto by guardian (if configured).

**Parameters:**
- `proposal_id` - Proposal to veto

**Requirements:**
- Caller must be guardian
- Proposal must not be executed
- Veto power must be enabled

**Events:**
- `ProposalVetoed { proposal_id, guardian }`

**Use Case:** Protect against malicious proposals, emergency response

## Query Functions

### Get Proposal State

```rust
fn get_proposal_state(proposal_id: u64) -> ProposalState
```

Returns the current state of a proposal.

**Returns:** Proposal state

```rust
enum ProposalState {
    Pending,      // Before voting starts
    Active,       // Voting in progress
    Canceled,     // Cancelled by proposer
    Defeated,     // Failed to reach quorum or majority
    Succeeded,    // Passed, ready to queue
    Queued,       // In timelock
    Expired,      // Timelock expired without execution
    Executed,     // Successfully executed
    Vetoed,       // Vetoed by guardian
}
```

### Get Votes

```rust
fn get_votes(account: String, block_number: u64) -> u64
```

Returns voting power at a specific block (historical snapshot).

**Parameters:**
- `account` - Address to query
- `block_number` - Historical block number

**Returns:** Voting power at that block

**Use Case:** Prevent vote buying, calculate voting power at proposal start

### Get Proposal

```rust
fn get_proposal(proposal_id: u64) -> Proposal
```

Returns full proposal details.

**Returns:** Proposal struct

```rust
struct Proposal {
    proposer: String,
    targets: Vec<String>,
    values: Vec<u64>,
    calldatas: Vec<Vec<u8>>,
    description: String,
    vote_start: u64,
    vote_end: u64,
    votes_for: u64,
    votes_against: u64,
    votes_abstain: u64,
    state: ProposalState,
}
```

### Get Proposal Votes

```rust
fn get_proposal_votes(proposal_id: u64) -> (u64, u64, u64)
```

Returns vote tallies for a proposal.

**Returns:** `(for_votes, against_votes, abstain_votes)`

### Has Voted

```rust
fn has_voted(proposal_id: u64, account: String) -> bool
```

Checks if an account has voted on a proposal.

**Returns:** True if voted

### Get Quorum

```rust
fn get_quorum(block_number: u64) -> u64
```

Returns required quorum at a specific block.

**Returns:** Minimum votes needed for proposal to pass

### Get Voting Power

```rust
fn get_voting_power(account: String) -> u64
```

Returns current voting power (including delegations).

**Returns:** Total voting power

### Get Delegates

```rust
fn get_delegate(account: String) -> String
```

Returns who an account has delegated to.

**Returns:** Delegatee address (or self if no delegation)

## Events

```rust
// Emitted when governor is initialized
event GovernorInitialized {
    voting_token: String,
    timelock: String,
    voting_delay: u64,
    voting_period: u64,
    proposal_threshold: u64,
    quorum_percentage: u64,
}

// Emitted when proposal is created
event ProposalCreated {
    proposal_id: u64,
    proposer: String,
    targets: Vec<String>,
    values: Vec<u64>,
    signatures: Vec<String>,
    calldatas: Vec<Vec<u8>>,
    vote_start: u64,
    vote_end: u64,
    description: String,
}

// Emitted when vote is cast
event VoteCast {
    voter: String,
    proposal_id: u64,
    support: VoteType,
    weight: u64,
    reason: String,
}

// Emitted when proposal is queued
event ProposalQueued {
    proposal_id: u64,
    eta: u64,
}

// Emitted when proposal is executed
event ProposalExecuted {
    proposal_id: u64,
}

// Emitted when proposal is cancelled
event ProposalCancelled {
    proposal_id: u64,
}

// Emitted when proposal is vetoed
event ProposalVetoed {
    proposal_id: u64,
    guardian: String,
}

// Emitted when voting power is delegated
event DelegateChanged {
    delegator: String,
    from_delegate: String,
    to_delegate: String,
}

// Emitted when delegate vote count changes
event DelegateVotesChanged {
    delegate: String,
    previous_balance: u64,
    new_balance: u64,
}
```

## Storage Layout

```rust
// Governance parameters
String: "voting_token"
String: "timelock"
u64: "voting_delay"
u64: "voting_period"
u64: "proposal_threshold"
u64: "quorum_percentage"
Option<String>: "guardian"

// Proposals
Map<u64, Proposal>: "proposals"
u64: "proposal_count"
Map<(u64, String), bool>: "has_voted"  // (proposal_id, voter) -> voted

// Vote tallies
Map<u64, ProposalVotes>: "proposal_votes"

// Delegation
Map<String, String>: "delegates"  // delegator -> delegatee
Map<String, u64>: "voting_power"  // account -> power (including delegations)

// Historical voting power (checkpoints)
Map<String, Vector<Checkpoint>>: "checkpoints"

struct Checkpoint {
    block_number: u64,
    votes: u64,
}

struct ProposalVotes {
    for_votes: u64,
    against_votes: u64,
    abstain_votes: u64,
}
```

## Security Considerations

### Vote Manipulation Prevention
- **Snapshot Voting**: Use historical balances at proposal creation
- **Voting Delay**: Prevent flash loan vote buying
- **Delegation Tracking**: Prevent double-counting delegations
- **Transfer Restrictions**: Lock tokens during voting (optional)

### Proposal Security
- **Timelock**: Delay execution for review period
- **Quorum**: Require minimum participation
- **Threshold**: Prevent spam proposals
- **Veto Power**: Emergency security mechanism

### Economic Security
- **Proposal Bond**: Require stake to propose (optional)
- **Vote Incentives**: Reward participation (optional)
- **Delegation Rewards**: Incentivize active delegates

### Governance Attacks
- **51% Attack**: Malicious majority governance
  - Mitigation: Timelock, guardian veto, high quorum
- **Vote Buying**: Purchase voting power
  - Mitigation: Snapshot voting, delegation lockup
- **Apathy**: Low participation
  - Mitigation: Lower quorum, incentives

## Example Usage

### Creating a Proposal

```rust
// Propose to send 10,000 CHERT from treasury for development grant

let proposal_id = governor.propose(
    vec![treasury_address],
    vec![10_000],
    vec![encode_call("transfer", recipient, 10_000)],
    "# Development Grant Q1 2026\n\n\
     Proposal to fund core development team.\n\n\
     ## Details\n\
     - Amount: 10,000 CHERT\n\
     - Recipient: Dev Team Multisig\n\
     - Duration: 3 months\n\n\
     ## Deliverables\n\
     1. Cross-shard bridge implementation\n\
     2. Privacy token features\n\
     3. Performance optimizations".to_string()
);

// Voting starts in 2 days (voting delay)
// Voting lasts 7 days (voting period)
```

### Voting on Proposal

```rust
// Alice votes in favor with explanation
governor.cast_vote_with_reason(
    proposal_id,
    VoteType::For,
    "This grant aligns with our roadmap and the team has delivered consistently.".to_string()
);

// Bob votes against
governor.cast_vote(proposal_id, VoteType::Against);

// Charlie abstains (counts for quorum only)
governor.cast_vote(proposal_id, VoteType::Abstain);
```

### Delegating Voting Power

```rust
// Alice delegates her voting power to Bob (trusted community member)
governor.delegate(bob_address);

// Bob now votes with Alice's tokens + his own
// Alice can reclaim by delegating to herself
governor.delegate(alice_address);  // Reclaim
```

### Executing Proposal

```rust
// After voting period ends and proposal succeeds:

// 1. Queue in timelock
governor.queue(proposal_id);
// Proposal enters 2-day timelock delay

// 2. Wait for timelock
// ... 2 days later ...

// 3. Execute
governor.execute(proposal_id);
// Treasury sends 10,000 CHERT to recipient
```

### Emergency Veto

```rust
// Guardian detects malicious proposal (e.g., drain treasury)
governor.veto(malicious_proposal_id);
// Proposal immediately cancelled
// Community can discuss and re-propose legitimate version
```

## Integration Examples

### Treasury Management

```rust
// Propose spending from protocol treasury
fn propose_grant(recipient: String, amount: u64, description: String) -> u64 {
    governor.propose(
        vec![treasury_contract],
        vec![0],
        vec![encode_call("transfer", recipient, amount)],
        description
    )
}
```

### Protocol Upgrades

```rust
// Propose contract upgrade through proxy
fn propose_upgrade(new_implementation: String) -> u64 {
    governor.propose(
        vec![proxy_contract],
        vec![0],
        vec![encode_call("upgrade_to", new_implementation)],
        "# Upgrade Protocol to v2.0\n\nSee upgrade documentation..."
    )
}
```

### Parameter Changes

```rust
// Propose changing protocol fee
fn propose_fee_change(new_fee: u64) -> u64 {
    governor.propose(
        vec![protocol_contract],
        vec![0],
        vec![encode_call("set_fee", new_fee)],
        format!("# Change Protocol Fee to {}%", new_fee / 100)
    )
}
```

### Multi-Action Proposals

```rust
// Propose multiple actions atomically
fn propose_ecosystem_funding() -> u64 {
    governor.propose(
        vec![
            treasury_contract,
            treasury_contract,
            treasury_contract,
        ],
        vec![5000, 3000, 2000],
        vec![
            encode_call("transfer", dev_team, 5000),
            encode_call("transfer", marketing, 3000),
            encode_call("transfer", security_audit, 2000),
        ],
        "# Q1 2026 Ecosystem Funding\n\n\
         Distribute 10,000 CHERT:\n\
         - 5,000 to development\n\
         - 3,000 to marketing\n\
         - 2,000 to security audit"
    )
}
```

## Governance Best Practices

### For Token Holders
- **Stay Informed**: Read proposals thoroughly before voting
- **Participate**: Vote on important decisions
- **Delegate Wisely**: Choose active, aligned delegates
- **Long-term Thinking**: Consider protocol sustainability

### For Proposers
- **Clear Description**: Use Markdown formatting, include rationale
- **Community Discussion**: Discuss in forum before proposing
- **Realistic Timelines**: Allow adequate voting time
- **Follow Template**: Use standard proposal format

### For Delegates
- **Be Active**: Vote on all proposals
- **Explain Decisions**: Use vote reasons for transparency
- **Represent Community**: Act in delegators' best interests
- **Stay Engaged**: Participate in discussions

## Differences from Compound/Uniswap Governance

- ✅ **Native Integration**: Built for Chert's architecture
- ✅ **Lower Costs**: Optimized gas usage
- ✅ **Guardian Veto**: Optional security layer
- ✅ **Flexible Quorum**: Dynamic based on participation
- ✅ **Snapshot Compatibility**: Can integrate with off-chain voting

## Testing Checklist

- [ ] Initialize governor with valid parameters
- [ ] Create proposal with sufficient tokens
- [ ] Reject proposal below threshold
- [ ] Vote during active period
- [ ] Reject vote outside voting period
- [ ] Reach quorum and pass proposal
- [ ] Fail proposal below quorum
- [ ] Queue successful proposal
- [ ] Execute after timelock delay
- [ ] Cancel proposal by proposer
- [ ] Delegate voting power
- [ ] Recursive delegation (A→B→C)
- [ ] Snapshot voting power at proposal start
- [ ] Emergency veto by guardian
- [ ] Test with maximum proposal size
- [ ] Verify vote counting accuracy

## License

MIT License - See LICENSE file for details

## References

- [Compound Governance](https://github.com/compound-finance/compound-protocol/tree/master/contracts/Governance)
- [OpenZeppelin Governor](https://docs.openzeppelin.com/contracts/governance)
- [Uniswap Governance](https://github.com/Uniswap/governance)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q1 2026
