# DEX AMM (Automated Market Maker)

A Uniswap V2-style automated market maker for decentralized token swaps on Chert Coin blockchain.

## Features

- ✅ **Constant Product Formula** - x * y = k liquidity model
- ✅ **Token Swaps** - Trade any CRC-20 token pair
- ✅ **Liquidity Provision** - Add/remove liquidity, earn trading fees
- ✅ **LP Tokens** - Represent share of liquidity pool
- ✅ **Price Oracle** - Time-weighted average price (TWAP)
- ✅ **Fee Distribution** - 0.3% trading fee to liquidity providers
- ✅ **Slippage Protection** - Minimum output amount guarantees
- ✅ **Deadline Protection** - Transaction expiry timestamps
- ✅ **Flash Swaps** - Borrow tokens within single transaction

## Use Cases

- 💱 **Token Trading** - Decentralized exchange without order books
- 💧 **Liquidity Mining** - Earn fees by providing liquidity
- 📊 **Price Discovery** - Market-driven token pricing
- 🔄 **Token Swaps** - Convert between any token pairs
- ⚡ **Flash Loans** - Uncollateralized loans within one transaction
- 🤖 **Arbitrage** - MEV opportunities across markets

## Architecture

### Liquidity Pool Model

```
Pool = { token0, token1, reserve0, reserve1, k }
k = reserve0 * reserve1  (constant product)

Price_0_in_1 = reserve1 / reserve0
Price_1_in_0 = reserve0 / reserve1
```

### Fee Structure

- **Trading Fee:** 0.3% of swap amount
- **Fee Distribution:** 100% to liquidity providers
- **Protocol Fee:** 0% (can be enabled by governance)

### Liquidity Provider Returns

```
LP_Share = liquidity_added / total_liquidity
Fee_Earned = trading_volume * 0.003 * LP_Share
Impermanent_Loss = price_divergence_risk
```

## API Reference

### Initialize

```rust
fn initialize(token0: String, token1: String)
```

Creates a new liquidity pool for a token pair.

**Parameters:**
- `token0` - First token contract address
- `token1` - Second token contract address

**Requirements:**
- Tokens must be different
- Pool must not already exist for this pair
- Token addresses must be valid CRC-20 contracts

**Events:**
- `PoolCreated { token0, token1, pool_address }`

### Add Liquidity

```rust
fn add_liquidity(
    token0_amount: u64,
    token1_amount: u64,
    min_token0: u64,
    min_token1: u64,
    deadline: u64
) -> (u64, u64, u64)
```

Adds liquidity to the pool and mints LP tokens.

**Parameters:**
- `token0_amount` - Desired amount of token0
- `token1_amount` - Desired amount of token1
- `min_token0` - Minimum token0 to add (slippage protection)
- `min_token1` - Minimum token1 to add (slippage protection)
- `deadline` - Unix timestamp transaction must execute by

**Returns:** `(actual_token0, actual_token1, liquidity_minted)`

**Requirements:**
- Must approve this contract to spend both tokens
- Amounts must maintain current pool ratio (after first deposit)
- Deadline must not be passed
- Minimum amounts must be satisfied

**Events:**
- `LiquidityAdded { provider, token0_amount, token1_amount, liquidity }`
- `Transfer { from: 0x0, to: provider, amount: liquidity }` (LP tokens)

### Remove Liquidity

```rust
fn remove_liquidity(
    liquidity: u64,
    min_token0: u64,
    min_token1: u64,
    deadline: u64
) -> (u64, u64)
```

Burns LP tokens and withdraws underlying tokens.

**Parameters:**
- `liquidity` - Amount of LP tokens to burn
- `min_token0` - Minimum token0 to receive
- `min_token1` - Minimum token1 to receive
- `deadline` - Transaction deadline

**Returns:** `(token0_amount, token1_amount)`

**Requirements:**
- Caller must have sufficient LP token balance
- Minimum amounts must be satisfied
- Deadline must not be passed

**Events:**
- `LiquidityRemoved { provider, token0_amount, token1_amount, liquidity }`
- `Transfer { from: provider, to: 0x0, amount: liquidity }` (LP tokens burned)

### Swap Exact Tokens For Tokens

```rust
fn swap_exact_tokens_for_tokens(
    amount_in: u64,
    min_amount_out: u64,
    path: Vec<String>,
    deadline: u64
) -> Vec<u64>
```

Swaps an exact amount of input tokens for output tokens.

**Parameters:**
- `amount_in` - Exact amount of input tokens to swap
- `min_amount_out` - Minimum output tokens to receive
- `path` - Array of token addresses representing swap path
- `deadline` - Transaction deadline

**Returns:** Array of amounts for each step in path

**Requirements:**
- Must approve this contract to spend input token
- Path length >= 2
- Sufficient output amount after fees and slippage
- Deadline must not be passed

**Events:**
- `Swap { sender, amount_in, amount_out, token_in, token_out }`
- `Sync { reserve0, reserve1 }` for each pool

### Swap Tokens For Exact Tokens

```rust
fn swap_tokens_for_exact_tokens(
    amount_out: u64,
    max_amount_in: u64,
    path: Vec<String>,
    deadline: u64
) -> Vec<u64>
```

Swaps tokens to receive an exact amount of output tokens.

**Parameters:**
- `amount_out` - Exact amount of output tokens desired
- `max_amount_in` - Maximum input tokens willing to pay
- `path` - Array of token addresses representing swap path
- `deadline` - Transaction deadline

**Returns:** Array of amounts for each step in path

**Requirements:**
- Must approve this contract to spend input token
- Input amount must not exceed maximum
- Path length >= 2
- Deadline must not be passed

### Flash Swap

```rust
fn flash_swap(
    amount0_out: u64,
    amount1_out: u64,
    to: String,
    data: Vec<u8>
)
```

Borrows tokens and calls recipient contract in same transaction.

**Parameters:**
- `amount0_out` - Amount of token0 to borrow
- `amount1_out` - Amount of token1 to borrow
- `to` - Recipient contract address
- `data` - Callback data passed to recipient

**Requirements:**
- Recipient must implement `onFlashSwap(sender, amount0, amount1, fee, data)`
- Recipient must repay borrowed amount + 0.3% fee
- At least one output amount must be > 0
- Pool reserves must be sufficient

**Events:**
- `FlashSwap { borrower, token0_amount, token1_amount, fee }`
- `Sync { reserve0, reserve1 }`

## Query Functions

### Get Reserves

```rust
fn get_reserves() -> (u64, u64, u64)
```

Returns current pool reserves and last update timestamp.

**Returns:** `(reserve0, reserve1, block_timestamp_last)`

### Get Amount Out

```rust
fn get_amount_out(amount_in: u64, reserve_in: u64, reserve_out: u64) -> u64
```

Calculates output amount for given input (including 0.3% fee).

**Formula:**
```
amount_in_with_fee = amount_in * 997
numerator = amount_in_with_fee * reserve_out
denominator = (reserve_in * 1000) + amount_in_with_fee
amount_out = numerator / denominator
```

**Returns:** Output amount

### Get Amount In

```rust
fn get_amount_in(amount_out: u64, reserve_in: u64, reserve_out: u64) -> u64
```

Calculates required input amount for desired output (including 0.3% fee).

**Formula:**
```
numerator = reserve_in * amount_out * 1000
denominator = (reserve_out - amount_out) * 997
amount_in = (numerator / denominator) + 1
```

**Returns:** Required input amount

### Get Amounts Out

```rust
fn get_amounts_out(amount_in: u64, path: Vec<String>) -> Vec<u64>
```

Calculates output amounts for multi-hop swap path.

**Returns:** Array of amounts for each step

### Get Amounts In

```rust
fn get_amounts_in(amount_out: u64, path: Vec<String>) -> Vec<u64>
```

Calculates required input amounts for multi-hop swap path.

**Returns:** Array of amounts for each step (reversed)

### Price Oracle

```rust
fn get_price_average(token: String, seconds_ago: u64) -> u128
```

Returns time-weighted average price over specified period.

**Parameters:**
- `token` - Token address to get price for
- `seconds_ago` - Number of seconds to average over

**Returns:** TWAP as fixed-point Q112.112 number

## Events

```rust
// Emitted when liquidity is added
event LiquidityAdded {
    provider: String,
    token0_amount: u64,
    token1_amount: u64,
    liquidity: u64,
}

// Emitted when liquidity is removed
event LiquidityRemoved {
    provider: String,
    token0_amount: u64,
    token1_amount: u64,
    liquidity: u64,
}

// Emitted on every swap
event Swap {
    sender: String,
    amount0_in: u64,
    amount1_in: u64,
    amount0_out: u64,
    amount1_out: u64,
    to: String,
}

// Emitted when reserves change
event Sync {
    reserve0: u64,
    reserve1: u64,
}

// Emitted when pool is created
event PoolCreated {
    token0: String,
    token1: String,
    pool: String,
}
```

## Storage Layout

```rust
// Pool reserves
u64: "reserve0"
u64: "reserve1"
u64: "block_timestamp_last"

// Token addresses
String: "token0"
String: "token1"

// LP token state (implements CRC-20)
u64: "total_supply"
Map<String, u64>: "balances"
Map<(String, String), u64>: "allowances"

// Price oracle cumulative prices
u128: "price0_cumulative_last"
u128: "price1_cumulative_last"

// Protocol parameters
u64: "fee_numerator"      // 3 (0.3%)
u64: "fee_denominator"    // 1000
u64: "minimum_liquidity"  // 1000 (burned on first deposit)
```

## Mathematical Details

### Constant Product Formula

```
x * y = k

Where:
- x = reserve of token0
- y = reserve of token1  
- k = constant product

When trading:
(x + Δx * 0.997) * (y - Δy) = k
```

### Liquidity Calculation

```
First deposit:
liquidity = sqrt(amount0 * amount1) - MINIMUM_LIQUIDITY

Subsequent deposits:
liquidity = min(
    (amount0 * total_supply) / reserve0,
    (amount1 * total_supply) / reserve1
)
```

### Withdrawal Calculation

```
amount0 = (liquidity * reserve0) / total_supply
amount1 = (liquidity * reserve1) / total_supply
```

### Price Impact

```
price_impact = 1 - (actual_price / expected_price)
              = 1 - ((amount_out * reserve_in) / (amount_in * reserve_out))
```

## Security Considerations

### Reentrancy Protection
- Locks on all state-changing functions
- State updates before external token transfers
- Checks-effects-interactions pattern

### Price Manipulation Resistance
- TWAP oracle for historical prices
- Minimum liquidity burn (1000 wei) prevents pool drainage
- Block timestamp tracking prevents same-block manipulation

### Slippage Protection
- Required minimum output amounts
- Required maximum input amounts
- Deadline enforcement prevents stale transactions

### Integer Safety
- Overflow checks on all arithmetic
- Q112.112 fixed-point for price calculations
- Minimum liquidity prevents division by zero

### Flash Loan Safety
- Repayment verified before state changes committed
- Fee collected on all flash swaps (0.3%)
- Callback data allows recipient verification

## Impermanent Loss

Liquidity providers face impermanent loss when token prices diverge:

```
IL = (2 * sqrt(price_ratio)) / (1 + price_ratio) - 1

Example:
- Token A doubles in price relative to B
- Price ratio = 2.0
- IL = -5.72%

This loss is "impermanent" because it disappears if prices return to original ratio.
Trading fees offset IL over time.
```

## Example Usage

### Creating a Pool

```rust
// Deploy DEX contract
let dex = deploy_dex_amm();

// Initialize CHERT/USDC pool
dex.initialize(
    "chert_token_address".to_string(),
    "usdc_token_address".to_string()
);
```

### Adding Liquidity

```rust
// Approve tokens
chert_token.approve(dex_address, 1000);
usdc_token.approve(dex_address, 5000);

// Add liquidity (1 CHERT = 5 USDC)
let (amount0, amount1, liquidity) = dex.add_liquidity(
    1000,  // 1000 CHERT
    5000,  // 5000 USDC
    950,   // Min CHERT (5% slippage)
    4750,  // Min USDC (5% slippage)
    deadline
);
```

### Swapping Tokens

```rust
// Approve input token
chert_token.approve(dex_address, 100);

// Swap 100 CHERT for USDC (minimum 450 USDC)
let amounts = dex.swap_exact_tokens_for_tokens(
    100,  // Exact 100 CHERT in
    450,  // Minimum 450 USDC out
    vec!["chert_address".to_string(), "usdc_address".to_string()],
    deadline
);
```

### Multi-Hop Swaps

```rust
// Swap CHERT -> USDC -> BTC
let path = vec![
    "chert_address".to_string(),
    "usdc_address".to_string(),
    "btc_address".to_string()
];

let amounts = dex.swap_exact_tokens_for_tokens(
    1000,  // 1000 CHERT
    10,    // Minimum 10 BTC
    path,
    deadline
);
```

## Integration Examples

### Arbitrage Bot

```rust
// Check price difference between DEXs
let price_dex1 = dex1.get_amount_out(1000, reserve0, reserve1);
let price_dex2 = dex2.get_amount_out(1000, reserve0, reserve1);

if price_dex2 > price_dex1 * 1.01 {  // 1% profit threshold
    // Buy on DEX1, sell on DEX2
    dex1.swap_exact_tokens_for_tokens(...);
    dex2.swap_exact_tokens_for_tokens(...);
}
```

### Liquidity Mining

```rust
// Add liquidity and track LP tokens
let liquidity = dex.add_liquidity(...);

// Stake LP tokens in farming contract
farming_contract.stake(liquidity);

// Earn CHERT rewards over time
// ...

// Unstake and remove liquidity
farming_contract.unstake(liquidity);
dex.remove_liquidity(liquidity, ...);
```

## Differences from Uniswap V2

- ✅ **Native Sharding** - Supports cross-shard token swaps
- ✅ **Lower Fees** - Optimized compute unit costs
- ✅ **Post-Quantum** - Compatible with Dilithium signatures
- ✅ **Built-in Oracle** - No separate oracle contract needed

## Testing Checklist

- [ ] Pool initialization with token pairs
- [ ] First liquidity deposit (minimum liquidity burn)
- [ ] Subsequent liquidity deposits (ratio maintenance)
- [ ] Liquidity withdrawal (proportional amounts)
- [ ] Token swaps (exact input)
- [ ] Token swaps (exact output)
- [ ] Multi-hop swaps through multiple pools
- [ ] Slippage protection enforcement
- [ ] Deadline expiry handling
- [ ] Flash swaps with repayment
- [ ] Price oracle TWAP calculation
- [ ] Reentrancy attack resistance
- [ ] Integer overflow protection
- [ ] Minimum liquidity lock

## License

MIT License - See LICENSE file for details

## References

- [Uniswap V2 Whitepaper](https://uniswap.org/whitepaper.pdf)
- [Uniswap V2 Core](https://github.com/Uniswap/v2-core)
- [Constant Product Formula](https://docs.uniswap.org/protocol/V2/concepts/protocol-overview/how-uniswap-works)

## Status

🚧 **In Development** - Implementation in progress

**Estimated Completion:** Q1 2026
