# Staking Pool

A liquid staking pool contract for delegating CHERT tokens to validators and earning staking rewards on Chert Coin blockchain.

## Features

- ✅ **Liquid Staking** - Receive tradeable stCHERT tokens representing staked CHERT
- ✅ **Automated Delegation** - Pool distributes stake across multiple validators
- ✅ **Reward Distribution** - Automatic compounding and claiming
- ✅ **Unstaking Queue** - Fair withdrawal with unbonding period
- ✅ **Validator Selection** - Performance-based validator optimization
- ✅ **Fee Management** - Configurable pool fees for operators
- ✅ **Emergency Pause** - Safety mechanism for critical issues
- ✅ **Governance Integration** - stCHERT holders retain voting power

## Use Cases

- 💰 **Passive Income** - Earn staking rewards without running validator
- 💱 **DeFi Collateral** - Use stCHERT in lending, DEX, and other protocols
- 📊 **Portfolio Diversification** - Stake distributed across validators
- 🔄 **Liquidity** - Trade staked position without unbonding wait
- 🏦 **Institutional Staking** - Large-scale delegation management
- 🎯 **Optimized Returns** - Automated validator selection for best yields

## Architecture

### Staking Model

```
User Flow:
1. Deposit CHERT → Receive stCHERT (1:1 initially)
2. Pool stakes CHERT with validators
3. Earn rewards → stCHERT value increases
4. Request unstake → Enter unbonding queue
5. Wait unbonding period → Withdraw CHERT

Exchange Rate:
stCHERT_value = total_staked_CHERT / total_stCHERT_supply
As rewards accumulate, each stCHERT represents more CHERT
```

### Validator Distribution

```
Pool distributes stake across validators based on:
- Performance (uptime, block production)
- Commission rates
- Total stake (avoid over-delegation)
- Slashing history
- Geographic diversity

Target: 20-50 validators
Rebalancing: Weekly automatic optimization
```

## API Reference

### Initialize

```rust
fn initialize(
    admin: String,
    fee_rate: u64,
    unbonding_period: u64
)
```

Initializes the staking pool.

**Parameters:**
- `admin` - Pool administrator address
- `fee_rate` - Pool fee as basis points (e.g., 1000 = 10%)
- `unbonding_period` - Days required for unstaking (e.g., 14)

**Requirements:**
- Fee rate must be ≤ 20% (2000 basis points)
- Unbonding period must match network parameter
- Admin must be valid address

**Events:**
- `PoolInitialized { admin, fee_rate, unbonding_period }`

### Stake

```rust
fn stake(amount: u64) -> u64
```

Stakes CHERT tokens and receives stCHERT.

**Parameters:**
- `amount` - Amount of CHERT to stake

**Returns:** Amount of stCHERT minted

**Requirements:**
- Amount must be ≥ minimum stake (e.g., 10 CHERT)
- Pool must not be paused
- Caller must have sufficient CHERT balance

**Events:**
- `Staked { user, chert_amount, stchert_minted, exchange_rate }`

**Calculation:**
```rust
if first_deposit {
    stchert = chert_amount  // 1:1 initially
} else {
    stchert = (chert_amount * total_stchert) / total_staked_chert
}
```

### Request Unstake

```rust
fn request_unstake(stchert_amount: u64) -> u64
```

Requests to unstake stCHERT (enters unbonding queue).

**Parameters:**
- `stchert_amount` - Amount of stCHERT to burn

**Returns:** Unstake request ID

**Requirements:**
- Caller must have sufficient stCHERT balance
- Amount must be ≥ minimum (e.g., 1 stCHERT)

**Events:**
- `UnstakeRequested { user, stchert_amount, chert_value, request_id, completion_time }`

**Process:**
1. Burns stCHERT immediately
2. Calculates CHERT to return based on exchange rate
3. Adds to unbonding queue
4. Returns after unbonding period

### Withdraw Unstaked

```rust
fn withdraw_unstaked(request_id: u64)
```

Withdraws CHERT after unbonding period completes.

**Parameters:**
- `request_id` - Unstake request identifier

**Requirements:**
- Request must belong to caller
- Unbonding period must be complete
- Request must not already be withdrawn

**Events:**
- `Withdrawn { user, request_id, chert_amount }`

### Claim Rewards

```rust
fn claim_rewards() -> u64
```

Claims accumulated staking rewards for caller.

**Returns:** CHERT rewards claimed

**Requirements:**
- Caller must have stCHERT balance
- Rewards must be available

**Events:**
- `RewardsClaimed { user, amount }`

**Note:** Rewards automatically compound in stCHERT value, this function is optional

### Delegate to Validator

```rust
fn delegate_to_validator(validator: String, amount: u64)
```

Delegates pool funds to a specific validator (admin only).

**Parameters:**
- `validator` - Validator address
- `amount` - CHERT amount to delegate

**Requirements:**
- Caller must be admin
- Validator must be active
- Pool must have sufficient undelegated balance
- Validator must meet pool criteria

**Events:**
- `Delegated { validator, amount, total_delegated }`

### Undelegate from Validator

```rust
fn undelegate_from_validator(validator: String, amount: u64)
```

Removes delegation from a validator (admin only).

**Parameters:**
- `validator` - Validator address
- `amount` - CHERT amount to undelegate

**Requirements:**
- Caller must be admin
- Delegation must exist
- Amount must be ≤ current delegation

**Events:**
- `Undelegated { validator, amount, reason }`

**Use Cases:**
- Validator performance degradation
- Over-delegation to single validator
- Validator commission increase
- Pool rebalancing

### Update Fee Rate

```rust
fn update_fee_rate(new_rate: u64)
```

Updates the pool fee rate (governance only).

**Parameters:**
- `new_rate` - New fee rate in basis points

**Requirements:**
- Must be called via governance
- New rate must be ≤ maximum (20%)
- Cannot decrease more than 5% per month (fairness)

**Events:**
- `FeeRateUpdated { old_rate, new_rate }`

### Pause/Unpause

```rust
fn pause()
fn unpause()
```

Emergency pause/unpause of pool operations.

**Requirements:**
- Caller must be admin or guardian
- Can only pause for valid reasons (security, emergency)

**Events:**
- `Paused { reason, paused_by }`
- `Unpaused { unpaused_by }`

**Effects:**
- Paused: No new stakes, unstake requests continue
- Unpaused: Normal operations resume

## Query Functions

### Get Exchange Rate

```rust
fn get_exchange_rate() -> (u64, u64)
```

Returns the current stCHERT to CHERT exchange rate.

**Returns:** `(total_staked_chert, total_stchert_supply)`

**Calculation:**
```rust
chert_per_stchert = total_staked_chert / total_stchert_supply
```

### Get User Balance

```rust
fn get_user_balance(user: String) -> (u64, u64)
```

Returns user's stCHERT balance and equivalent CHERT value.

**Returns:** `(stchert_balance, chert_value)`

### Get Unstake Request

```rust
fn get_unstake_request(request_id: u64) -> UnstakeRequest
```

Returns details of an unstake request.

**Returns:** Unstake request details

```rust
struct UnstakeRequest {
    user: String,
    stchert_amount: u64,
    chert_amount: u64,
    request_time: u64,
    completion_time: u64,
    status: RequestStatus,
}
```

### Get Pending Unstakes

```rust
fn get_pending_unstakes(user: String) -> Vec<UnstakeRequest>
```

Returns all pending unstake requests for a user.

**Returns:** Array of unstake requests

### Get Pool Stats

```rust
fn get_pool_stats() -> PoolStats
```

Returns comprehensive pool statistics.

**Returns:** Pool statistics

```rust
struct PoolStats {
    total_staked_chert: u64,
    total_stchert_supply: u64,
    total_users: u64,
    validator_count: u64,
    apr: u64,  // Annual percentage rate (basis points)
    fee_rate: u64,
    tvl: u64,  // Total value locked
}
```

### Get Validator Delegations

```rust
fn get_validator_delegations() -> Vec<(String, u64)>
```

Returns all validator delegations and amounts.

**Returns:** Array of (validator_address, delegated_amount)

### Calculate Rewards

```rust
fn calculate_rewards(user: String) -> u64
```

Calculates pending rewards for a user.

**Returns:** Estimated reward amount

### Get APR

```rust
fn get_apr() -> u64
```

Returns the current annual percentage rate.

**Returns:** APR in basis points (e.g., 1200 = 12%)

**Calculation:**
```rust
apr = (annual_rewards / total_staked) * 10000
```

## Events

```rust
// Emitted when pool is initialized
event PoolInitialized {
    admin: String,
    fee_rate: u64,
    unbonding_period: u64,
}

// Emitted when user stakes
event Staked {
    user: String,
    chert_amount: u64,
    stchert_minted: u64,
    exchange_rate: u64,
    timestamp: u64,
}

// Emitted when unstake is requested
event UnstakeRequested {
    user: String,
    request_id: u64,
    stchert_amount: u64,
    chert_value: u64,
    completion_time: u64,
}

// Emitted when funds are withdrawn
event Withdrawn {
    user: String,
    request_id: u64,
    chert_amount: u64,
}

// Emitted when rewards are claimed
event RewardsClaimed {
    user: String,
    amount: u64,
}

// Emitted when pool delegates to validator
event Delegated {
    validator: String,
    amount: u64,
    total_delegated: u64,
}

// Emitted when pool undelegates from validator
event Undelegated {
    validator: String,
    amount: u64,
    reason: String,
}

// Emitted when rewards are distributed
event RewardsDistributed {
    epoch: u64,
    total_rewards: u64,
    pool_fee: u64,
    user_rewards: u64,
}

// Emitted when fee rate changes
event FeeRateUpdated {
    old_rate: u64,
    new_rate: u64,
}

// Emitted when pool is paused
event Paused {
    reason: String,
    paused_by: String,
}

// Emitted when pool is unpaused
event Unpaused {
    unpaused_by: String,
}
```

## Storage Layout

```rust
// Pool state
u64: "total_staked_chert"
u64: "total_stchert_supply"
u64: "total_rewards_accumulated"

// User balances (stCHERT implements CRC-20)
Map<String, u64>: "stchert_balances"
Map<String, u64>: "user_rewards"

// Unstaking queue
Map<u64, UnstakeRequest>: "unstake_requests"
Map<String, Vector<u64>>: "user_unstake_requests"
u64: "next_request_id"

// Validator delegations
Map<String, u64>: "validator_delegations"  // validator -> amount
Vector<String>: "active_validators"

// Configuration
String: "admin"
u64: "fee_rate"  // Basis points
u64: "unbonding_period"  // Days
u64: "min_stake_amount"  // 10 CHERT
u64: "min_validator_stake"  // 1000 CHERT
bool: "paused"

// Performance tracking
Map<String, ValidatorPerformance>: "validator_performance"
u64: "last_rebalance_time"

struct ValidatorPerformance {
    uptime: u64,  // Basis points
    commission: u64,
    total_stake: u64,
    slashing_events: u64,
}
```

## Security Considerations

### Economic Security
- **Validator Diversification**: Spread risk across many validators
- **Performance Monitoring**: Track validator uptime and efficiency
- **Slashing Protection**: Automatic undelegation from slashed validators
- **Fee Limits**: Maximum 20% fee protects users

### Smart Contract Security
- **Reentrancy Protection**: Guards on all state-changing functions
- **Overflow Protection**: Safe math for all calculations
- **Access Control**: Admin functions properly restricted
- **Emergency Pause**: Circuit breaker for critical issues

### Exchange Rate Security
- **Monotonic Increase**: Exchange rate only increases (compounding)
- **Rounding Protection**: Favor users in rounding decisions
- **Manipulation Resistance**: Large pool size prevents rate manipulation

### Unbonding Security
- **Fair Ordering**: FIFO queue for unstake requests
- **Rate Limiting**: Prevent bank run scenarios
- **Reserve Management**: Maintain liquidity for withdrawals

## Example Usage

### Staking CHERT

```rust
// Alice stakes 1000 CHERT
let stchert_received = staking_pool.stake(1000);

// Alice receives stCHERT tokens (initially 1:1, ratio improves over time)
// She can now use stCHERT in DeFi protocols
```

### Unstaking After Rewards

```rust
// 1 year later, stCHERT exchange rate is 1:1.12 (12% APR)
// Alice wants to unstake

// Check her current value
let (stchert_balance, chert_value) = staking_pool.get_user_balance(alice);
// stchert_balance = 1000
// chert_value = 1120 (she earned 120 CHERT in rewards)

// Request unstake
let request_id = staking_pool.request_unstake(1000);

// Wait unbonding period (14 days)
// ...

// Withdraw
staking_pool.withdraw_unstaked(request_id);
// Alice receives 1120 CHERT (original 1000 + 120 rewards)
```

### Using stCHERT in DeFi

```rust
// Bob stakes CHERT, receives stCHERT
staking_pool.stake(5000);  // Receive 5000 stCHERT

// Use stCHERT as collateral in lending protocol
lending_protocol.deposit_collateral(stchert_token, 5000);

// Borrow against stCHERT while still earning staking rewards
lending_protocol.borrow(usdc_token, 20000);

// stCHERT value keeps increasing while used as collateral
```

### Pool Administration

```rust
// Admin adds high-performing validator
staking_pool.delegate_to_validator(
    "validator_xyz",
    100_000  // Delegate 100k CHERT
);

// Remove underperforming validator
staking_pool.undelegate_from_validator(
    "validator_abc",
    50_000  // Undelegate 50k CHERT
);

// Rebalance automatically based on performance
staking_pool.rebalance_validators();
```

## Integration Examples

### Lending Protocol

```rust
// Accept stCHERT as collateral
fn get_collateral_value(user: String) -> u64 {
    let stchert_balance = stchert_token.balance_of(user);
    let (total_staked, total_supply) = staking_pool.get_exchange_rate();
    let chert_value = (stchert_balance * total_staked) / total_supply;
    
    // Apply haircut for liquidation buffer (90% LTV)
    chert_value * 9 / 10
}
```

### DEX Integration

```rust
// Trade stCHERT/CHERT pair on DEX
// Arbitrage keeps price close to exchange rate

fn arbitrage_opportunity() -> bool {
    let dex_price = dex.get_price("stCHERT/CHERT");
    let (total_staked, total_supply) = staking_pool.get_exchange_rate();
    let fair_price = total_staked / total_supply;
    
    abs(dex_price - fair_price) > 100  // 1% deviation = arb opportunity
}
```

### Yield Aggregator

```rust
// Auto-compound staking rewards into other strategies
fn harvest_and_compound(user: String) {
    // User's stCHERT automatically appreciates
    // No need for active compounding
    
    // If user wants liquid rewards:
    let rewards = staking_pool.calculate_rewards(user);
    if rewards > threshold {
        staking_pool.claim_rewards();
        // Reinvest rewards elsewhere
    }
}
```

## Differences from Lido

- ✅ **Native Integration**: Built for Chert's consensus
- ✅ **Lower Fees**: Optimized for efficiency
- ✅ **Simpler Governance**: Focused core functionality
- ✅ **Shard Aware**: Works with Chert's sharding

## Testing Checklist

- [ ] Initialize pool with configuration
- [ ] Stake CHERT and receive stCHERT (1:1 initially)
- [ ] Stake again after rewards (improved rate)
- [ ] Request unstake
- [ ] Withdraw after unbonding period
- [ ] Claim rewards
- [ ] Delegate to multiple validators
- [ ] Undelegate from validators
- [ ] Calculate exchange rate correctly
- [ ] Handle reward distribution
- [ ] Apply pool fees correctly
- [ ] Pause pool operations
- [ ] Update fee rate via governance
- [ ] Test with maximum user count
- [ ] Simulate validator slashing
- [ ] Test unbonding queue fairness

## License

MIT License - See LICENSE file for details

## References

- [Lido Finance](https://lido.fi/)
- [Rocket Pool](https://rocketpool.net/)
- [Liquid Staking Whitepaper](https://lido.fi/static/Lido:Ethereum-Liquid-Staking.pdf)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q1 2026
