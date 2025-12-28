# Oracle Integration Contract

A decentralized oracle system for bringing off-chain data on-chain securely on Chert Coin blockchain.

## Features

- ✅ **Data Feed Aggregation** - Combine multiple oracle sources
- ✅ **Median Calculation** - Outlier-resistant price aggregation
- ✅ **Staleness Checks** - Reject outdated data automatically
- ✅ **Dispute Resolution** - Challenge and resolve incorrect data
- ✅ **Access Control** - Authorized data providers only
- ✅ **Historical Data** - Query past values and timestamps
- ✅ **Multiple Data Types** - Support prices, weather, sports, etc.
- ✅ **Economic Security** - Stake-based provider incentives

## Use Cases

- 💱 **Price Feeds** - Real-time asset prices for DeFi protocols
- 🌤️ **Weather Data** - Parametric insurance, agricultural contracts
- 🏆 **Sports Results** - Prediction markets, betting platforms
- 📊 **Financial Data** - Interest rates, indices, forex rates
- 🗳️ **Election Results** - Governance and prediction markets
- 🔗 **Cross-Chain Data** - Bridge state from other blockchains
- 📈 **Market Data** - Trading volume, liquidity metrics

## Architecture

### Oracle Model

```
Data Flow:
1. Off-Chain Source → Data Provider retrieves data
2. Data Provider → Submits signed data to oracle contract
3. Oracle Contract → Aggregates multiple submissions
4. Smart Contract → Queries aggregated data
5. Dispute Window → Challengers can dispute incorrect data
```

### Aggregation Methods

```
MEDIAN: Take middle value from sorted submissions
- Resistant to outliers
- Best for price feeds

AVERAGE: Mean of all submissions
- Smooth aggregation
- Sensitive to outliers

WEIGHTED_AVERAGE: Based on provider reputation/stake
- Rewards reliable providers
- More complex

LAST_REPORTER: Most recent submission wins
- Fast updates
- Less secure
```

## API Reference

### Initialize

```rust
fn initialize(
    aggregation_method: AggregationMethod,
    min_providers: u64,
    staleness_threshold: u64
)
```

Initializes the oracle contract.

**Parameters:**
- `aggregation_method` - How to combine multiple submissions
- `min_providers` - Minimum data providers required
- `staleness_threshold` - Maximum age for valid data (seconds)

**Requirements:**
- Min providers must be ≥ 1
- Staleness threshold must be reasonable (< 1 day for prices)

**Events:**
- `OracleInitialized { aggregation_method, min_providers, staleness_threshold }`

### Register Provider

```rust
fn register_provider(provider: String, stake: u64)
```

Registers a new data provider with economic stake.

**Parameters:**
- `provider` - Provider address
- `stake` - Amount of CHERT tokens to stake

**Requirements:**
- Caller must have admin role
- Stake must meet minimum requirement
- Provider must not already be registered

**Events:**
- `ProviderRegistered { provider, stake }`

### Submit Data

```rust
fn submit_data(
    feed_id: String,
    value: i128,
    timestamp: u64,
    signature: Vec<u8>
)
```

Submits oracle data from authorized provider.

**Parameters:**
- `feed_id` - Data feed identifier (e.g., "BTC/USD", "ETH/USD")
- `value` - Data value (price, temperature, etc.) as fixed-point integer
- `timestamp` - When data was observed off-chain
- `signature` - Provider's signature over (feed_id, value, timestamp)

**Requirements:**
- Caller must be registered provider
- Timestamp must be recent (within staleness threshold)
- Signature must be valid
- Cannot submit same timestamp twice

**Events:**
- `DataSubmitted { provider, feed_id, value, timestamp }`

### Get Latest Data

```rust
fn get_latest_data(feed_id: String) -> (i128, u64)
```

Returns the latest aggregated value for a feed.

**Parameters:**
- `feed_id` - Data feed identifier

**Returns:** `(value, timestamp)` of latest aggregated data

**Requirements:**
- Feed must exist
- Must have minimum provider submissions
- Data must not be stale

**Reverts if:**
- Insufficient data providers
- Data is stale
- No data available

### Get Historical Data

```rust
fn get_historical_data(feed_id: String, timestamp: u64) -> Option<i128>
```

Returns data value at specific past timestamp.

**Parameters:**
- `feed_id` - Data feed identifier
- `timestamp` - Historical timestamp to query

**Returns:** Data value if available, None otherwise

### Challenge Data

```rust
fn challenge_data(
    feed_id: String,
    timestamp: u64,
    reason: String,
    evidence: Vec<u8>
)
```

Challenges potentially incorrect oracle data.

**Parameters:**
- `feed_id` - Data feed being challenged
- `timestamp` - Specific data point timestamp
- `reason` - Human-readable explanation
- `evidence` - Off-chain proof of incorrectness

**Requirements:**
- Caller must stake challenge bond
- Data must be within dispute window
- Cannot challenge same data twice

**Events:**
- `DataChallenged { challenger, feed_id, timestamp, reason }`

### Resolve Dispute

```rust
fn resolve_dispute(
    challenge_id: u64,
    ruling: DisputeRuling
)
```

Resolves a data challenge (admin/governance only).

**Parameters:**
- `challenge_id` - Challenge to resolve
- `ruling` - Accept or Reject challenge

**Requirements:**
- Caller must have resolver role
- Challenge must be pending

**Effects:**
- If ACCEPT: Slash provider stake, reward challenger
- If REJECT: Return provider stake, slash challenger bond

**Events:**
- `DisputeResolved { challenge_id, ruling, slashed_party }`

### Update Aggregation Method

```rust
fn update_aggregation_method(method: AggregationMethod)
```

Changes how multiple submissions are aggregated.

**Parameters:**
- `method` - New aggregation method

**Requirements:**
- Must be called via governance
- Method must be valid

**Events:**
- `AggregationMethodUpdated { old_method, new_method }`

## Query Functions

### Is Provider

```rust
fn is_provider(address: String) -> bool
```

Checks if address is registered provider.

**Returns:** True if registered

### Get Provider Stake

```rust
fn get_provider_stake(provider: String) -> u64
```

Returns provider's staked amount.

**Returns:** Stake in CHERT tokens

### Get Provider Count

```rust
fn get_provider_count(feed_id: String) -> u64
```

Returns number of active providers for feed.

**Returns:** Provider count

### Get All Submissions

```rust
fn get_all_submissions(feed_id: String, timestamp: u64) -> Vec<Submission>
```

Returns all provider submissions for a specific timestamp.

**Returns:** Array of submissions

```rust
struct Submission {
    provider: String,
    value: i128,
    timestamp: u64,
    signature: Vec<u8>,
}
```

### Is Data Stale

```rust
fn is_data_stale(feed_id: String) -> bool
```

Checks if latest data exceeds staleness threshold.

**Returns:** True if stale

### Get Feed Info

```rust
fn get_feed_info(feed_id: String) -> FeedInfo
```

Returns metadata about a data feed.

**Returns:** Feed information

```rust
struct FeedInfo {
    name: String,
    description: String,
    data_type: DataType,
    decimals: u8,
    min_providers: u64,
    staleness_threshold: u64,
    total_updates: u64,
    last_update: u64,
}
```

## Events

```rust
// Emitted when oracle is initialized
event OracleInitialized {
    aggregation_method: AggregationMethod,
    min_providers: u64,
    staleness_threshold: u64,
}

// Emitted when provider is registered
event ProviderRegistered {
    provider: String,
    stake: u64,
}

// Emitted when provider is removed
event ProviderRemoved {
    provider: String,
    reason: String,
}

// Emitted when data is submitted
event DataSubmitted {
    provider: String,
    feed_id: String,
    value: i128,
    timestamp: u64,
}

// Emitted when aggregated data updates
event DataUpdated {
    feed_id: String,
    value: i128,
    timestamp: u64,
    provider_count: u64,
}

// Emitted when data is challenged
event DataChallenged {
    challenge_id: u64,
    challenger: String,
    feed_id: String,
    timestamp: u64,
    reason: String,
}

// Emitted when dispute is resolved
event DisputeResolved {
    challenge_id: u64,
    ruling: DisputeRuling,
    slashed_party: String,
    slashed_amount: u64,
}

// Emitted when provider is slashed
event ProviderSlashed {
    provider: String,
    amount: u64,
    reason: String,
}
```

## Storage Layout

```rust
// Provider management
Map<String, u64>: "provider_stakes"  // provider -> stake
Map<String, bool>: "is_provider"
Vector<String>: "providers"

// Data feeds
Map<String, FeedData>: "feeds"  // feed_id -> latest data
Map<(String, u64), Vec<Submission>>: "submissions"  // (feed_id, timestamp) -> submissions
Map<String, Vector<DataPoint>>: "historical_data"  // feed_id -> [past values]

// Configuration
AggregationMethod: "aggregation_method"
u64: "min_providers"
u64: "staleness_threshold"
u64: "min_stake_amount"
u64: "dispute_window"  // 24 hours
u64: "challenge_bond"  // 100 CHERT

// Challenges
Map<u64, Challenge>: "challenges"
u64: "challenge_counter"

struct FeedData {
    value: i128,
    timestamp: u64,
    provider_count: u64,
}

struct DataPoint {
    value: i128,
    timestamp: u64,
}

struct Challenge {
    challenger: String,
    provider: String,
    feed_id: String,
    timestamp: u64,
    reason: String,
    evidence: Vec<u8>,
    status: ChallengeStatus,
}
```

## Security Considerations

### Economic Security
- Providers must stake tokens to participate
- Slashing for submitting incorrect data
- Challenge bonds prevent spam disputes

### Data Validation
- Signature verification for all submissions
- Staleness checks prevent old data
- Minimum provider threshold prevents manipulation

### Aggregation Security
- Median aggregation resistant to outliers
- Weighted methods account for provider reputation
- Historical data immutable after dispute window

### Access Control
- Only registered providers can submit
- Admin role for provider management
- Governance for parameter updates

### Timestamp Security
- Block timestamp used for staleness checks
- Cannot submit future timestamps
- Historical queries use exact timestamps

## Example Usage

### Deploying Price Feed Oracle

```rust
let oracle = deploy_oracle();

oracle.initialize(
    AggregationMethod::Median,
    3,      // Require 3 providers minimum
    300     // 5-minute staleness threshold
);

// Register price feed providers
oracle.register_provider("provider1", 10000);
oracle.register_provider("provider2", 10000);
oracle.register_provider("provider3", 10000);
```

### Submitting Price Data

```rust
// Off-chain: Provider fetches BTC price from exchanges
let btc_price = fetch_btc_price();  // $45,000.00
let value = 4500000;  // $45k with 2 decimals precision
let timestamp = current_timestamp();

// Sign the data
let signature = sign(provider_key, (feed_id, value, timestamp));

// Submit to oracle
oracle.submit_data(
    "BTC/USD".to_string(),
    value,
    timestamp,
    signature
);
```

### Querying Oracle Data

```rust
// DeFi protocol queries BTC price
let (price, timestamp) = oracle.get_latest_data("BTC/USD".to_string());

// Verify freshness
require!(!oracle.is_data_stale("BTC/USD".to_string()), "Stale price");

// Use price for liquidation calculation
let collateral_value = collateral_amount * price / 100;  // Adjust for decimals
if collateral_value < debt_value * liquidation_ratio {
    liquidate(position);
}
```

### Challenging Incorrect Data

```rust
// Monitor detects incorrect price submission
// Provider submitted $50k when actual price was $45k

oracle.challenge_data(
    "BTC/USD".to_string(),
    suspicious_timestamp,
    "Price 10% above all other exchanges".to_string(),
    encode_evidence(exchange_prices)  // Proof from multiple exchanges
);

// Stake challenge bond
// Wait for dispute resolution
```

### Aggregating Multiple Feeds

```rust
// Get prices from multiple feeds
let btc_usd = oracle.get_latest_data("BTC/USD".to_string()).0;
let eth_usd = oracle.get_latest_data("ETH/USD".to_string()).0;
let eth_btc = oracle.get_latest_data("ETH/BTC".to_string()).0;

// Verify consistency (triangular arbitrage check)
let calculated_eth_btc = (eth_usd * 100) / btc_usd;
let difference = abs(calculated_eth_btc - eth_btc);
require!(difference < 100, "Price inconsistency detected");  // 1% tolerance
```

## Integration Examples

### DeFi Lending Protocol

```rust
fn calculate_collateral_ratio(user: String) -> u64 {
    let collateral = user_collateral[user];
    let debt = user_debt[user];
    
    // Query oracle for collateral token price
    let (price, _) = oracle.get_latest_data("COLLATERAL/USD".to_string());
    require!(!oracle.is_data_stale("COLLATERAL/USD".to_string()), "Stale price");
    
    let collateral_value = collateral * price / PRICE_DECIMALS;
    let ratio = (collateral_value * 10000) / debt;  // Basis points
    
    ratio
}

fn liquidate_if_undercollateralized(user: String) {
    let ratio = calculate_collateral_ratio(user);
    if ratio < LIQUIDATION_THRESHOLD {
        // Liquidate position
        execute_liquidation(user);
    }
}
```

### Parametric Insurance

```rust
fn check_insurance_trigger(policy_id: u64) {
    let policy = policies[policy_id];
    
    // Query weather oracle
    let (rainfall, timestamp) = oracle.get_latest_data(
        format!("RAINFALL/{}", policy.location)
    );
    
    // Check if rainfall exceeded trigger threshold
    if rainfall > policy.trigger_threshold {
        // Automatically pay out insurance
        pay_insurance_claim(policy_id, policy.payout_amount);
    }
}
```

### Prediction Market

```rust
fn resolve_market(market_id: u64, outcome_feed: String) {
    let market = markets[market_id];
    require!(block_timestamp() > market.resolution_time, "Too early");
    
    // Query oracle for outcome
    let (outcome, _) = oracle.get_latest_data(outcome_feed);
    
    // Distribute winnings based on outcome
    distribute_winnings(market_id, outcome);
    
    market.resolved = true;
}
```

## Differences from Chainlink

- ✅ **Native Integration** - Built into Chert blockchain
- ✅ **Lower Costs** - No LINK token required
- ✅ **Simpler Model** - Focused on essential oracle functionality
- ✅ **Stake-Based** - Economic security through CHERT staking
- ✅ **On-Chain Aggregation** - Transparent calculation

## Testing Checklist

- [ ] Initialize oracle with various configurations
- [ ] Register multiple data providers
- [ ] Submit data from multiple providers
- [ ] Aggregate data using median method
- [ ] Aggregate data using average method
- [ ] Query latest data
- [ ] Query historical data
- [ ] Reject stale data submissions
- [ ] Handle insufficient provider count
- [ ] Challenge incorrect data
- [ ] Resolve disputes (accept challenge)
- [ ] Resolve disputes (reject challenge)
- [ ] Slash dishonest providers
- [ ] Prevent unauthorized submissions
- [ ] Test with maximum provider count
- [ ] Verify signature validation

## License

MIT License - See LICENSE file for details

## References

- [Chainlink Architecture](https://docs.chain.link/architecture-overview)
- [Oracle Problem](https://blog.ethereum.org/2014/03/28/schellingcoin-a-minimal-trust-universal-data-feed)
- [Band Protocol](https://docs.bandchain.org/)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q2 2026
