---
title: Portfolio Tracking
sidebar_position: 1
---

# Portfolio Tracking

This document explains how Wealthfolio tracks investment portfolios across
multiple accounts.

## Overview

Wealthfolio's portfolio tracking system provides:

- **Multi-account support** - Track multiple investment accounts
- **Multiple asset types** - Stocks, ETFs, mutual funds, bonds, cash, crypto
- **FIFO cost basis** - Accurate cost calculation using First-In-First-Out
- **Real-time valuation** - Portfolio value based on current market prices
- **Currency conversion** - Multi-currency support with automatic conversion
- **Aggregation** - Total portfolio view across all accounts

## Core Concepts

### Accounts

Types of investment accounts supported:

| Account Type | Description          | Example               |
| ------------ | -------------------- | --------------------- |
| `brokerage`  | Brokerage account    | E\*TRADE, Fidelity    |
| `retirement` | Retirement account   | 401(k), IRA, Roth IRA |
| `cash`       | Cash or bank account | Savings, checking     |

**Account Schema**:

```rust
pub struct Account {
    pub id: i32,
    pub name: String,
    pub account_type: AccountType,
    pub currency: String,           // e.g., "USD", "VND"
    pub is_active: bool,
    pub sync_source: Option`String>`, // e.g., "Manual", "CSV Import"
    pub created_at: DateTime`Utc>`,
    pub updated_at: DateTime`Utc>`,
}
```

### Asset Types

Supported asset classifications:

| Asset Type   | Description              |
| ------------ | ------------------------ |
| `Stock`      | Individual stocks        |
| `Etf`        | Exchange-traded funds    |
| `MutualFund` | Mutual funds             |
| `Bond`       | Bonds and fixed income   |
| `Crypto`     | Cryptocurrencies         |
| `Cash`       | Cash equivalents         |
| `Commodity`  | Commodities (gold, etc.) |

**Asset Schema**:

```rust
pub struct Asset {
    pub id: i32,
    pub symbol: String,            // e.g., "AAPL", "VFMVF1"
    pub name: String,
    pub asset_type: AssetType,
    pub currency: String,           // Base currency for asset
    pub exchange: Option`String>`,    // Exchange listing
    pub data_source: DataSource,    // Yahoo, VCI, Manual, etc.
    pub isin: Option`String>`,      // ISIN identifier
    pub created_at: DateTime`Utc>`,
    pub updated_at: DateTime`Utc>`,
}
```

### Activity Types

Supported transaction types:

| Activity Type | Direction | Examples                          |
| ------------- | --------- | --------------------------------- |
| `BUY`         | Inflow    | Buying shares, ETFs, mutual funds |
| `SELL`        | Outflow   | Selling shares                    |
| `DIVIDEND`    | Inflow    | Dividend payments                 |
| `INTEREST`    | Inflow    | Interest income                   |
| `FEE`         | Outflow   | Transaction fees, commissions     |
| `DEPOSIT`     | Inflow    | Cash deposits to account          |
| `WITHDRAWAL`  | Outflow   | Cash withdrawals from account     |

**Activity Schema**:

```rust
pub struct Activity {
    pub id: i32,
    pub account_id: i32,
    pub asset_id: i32,
    pub activity_type: ActivityType,
    pub date: NaiveDate,
    pub quantity: Decimal,          // Number of shares/units
    pub unit_price: Decimal,        // Price per share/unit
    pub fee: Decimal,              // Transaction fee
    pub currency: String,           // Transaction currency
    pub notes: Option`String>`,
    pub imported_from: Option`String>`, // Source: "CSV Import", "Manual"
    pub created_at: DateTime`Utc>`,
    pub updated_at: DateTime`Utc>`,
}
```

---

## Holdings Calculation

### FIFO Algorithm

Wealthfolio uses **First-In-First-Out (FIFO)** to calculate holdings and cost
basis:

```
For each asset in account:
  1. Sort all BUY activities by date (oldest first)
  2. Sort all SELL activities by date
  3. For each SELL:
     a. Consume from oldest BUY until quantity satisfied
     b. Calculate weighted average cost
     c. Save holding snapshot
  4. Remaining BUYs become current holdings
```

### Implementation

```rust
pub struct PortfolioService {
    pub async fn calculate_holdings_snapshots(&self, account_id: i32) -> Result`Vec``HoldingSnapshot>`> {
        // 1. Get all activities for account
        let activities = self.activity_repo
            .search_activities(Some(account_id), None, None, None)
            .await?;

        // 2. Group by asset
        let mut asset_queues: HashMap`i32`, VecDeque`Activity>`> = HashMap::new();

        for activity in activities {
            if activity.activity_type == ActivityType::BUY {
                let queue = asset_queues
                    .entry(activity.asset_id)
                    .or_insert_with(VecDeque::new);
                queue.push_back(activity);
            }
        }

        // 3. Process SELL activities
        let mut snapshots = Vec::new();

        for activity in activities {
            if activity.activity_type == ActivityType::SELL {
                let queue = asset_queues.get_mut(&activity.asset_id)
                    .ok_or(Error::AssetNotFound)?;

                let mut remaining_sell_qty = activity.quantity;
                let mut total_cost = Decimal::ZERO;

                // Consume from oldest BUYs
                while remaining_sell_qty > Decimal::ZERO && !queue.is_empty() {
                    let oldest_buy = queue.front_mut()
                        .ok_or(Error::InsufficientHoldings)?;

                    let buy_qty = oldest_buy.quantity;

                    if buy_qty ``= remaining_sell_qty {
                        // Consume entire BUY
                        total_cost += buy_qty * oldest_buy.unit_price;
                        remaining_sell_qty -= buy_qty;
                        queue.pop_front();
                    } else {
                        // Consume partial BUY
                        let consumed_qty = remaining_sell_qty;
                        total_cost += consumed_qty * oldest_buy.unit_price;
                        oldest_buy.quantity -= consumed_qty;
                        remaining_sell_qty = Decimal::ZERO;
                    }
                }

                // Calculate average cost
                let avg_cost = total_cost / activity.quantity;

                // Save snapshot
                snapshots.push(HoldingSnapshot {
                    account_id,
                    asset_id: activity.asset_id,
                    quantity: activity.quantity,
                    avg_cost,
                    current_value: Decimal::ZERO,
                });
            }
        }

        // 4. Save remaining holdings (unsold positions)
        for (asset_id, queue) in asset_queues {
            for activity in queue {
                snapshots.push(HoldingSnapshot {
                    account_id,
                    asset_id,
                    quantity: activity.quantity,
                    avg_cost: activity.unit_price,
                    current_value: Decimal::ZERO,
                });
            }
        }

        Ok(snapshots)
    }
}
```

### Example Calculation

**Activities**:

| Date       | Type | Symbol | Quantity | Price |
| ---------- | ---- | ------ | -------- | ----- |
| 2024-01-01 | BUY  | AAPL   | 100      | $150  |
| 2024-02-01 | BUY  | AAPL   | 50       | $160  |
| 2024-03-01 | SELL | AAPL   | 75       | $170  |

**FIFO Processing**:

1. **BUY Queue**:
   - Position 1: 100 @ $150
   - Position 2: 50 @ $160

2. **Process SELL: 75 shares**:
   - Consume 75 from Position 1 (100 @ $150)
   - Cost: 75 × $150 = $11,250
   - Average Cost: $11,250 / 75 = $150

3. **Remaining Holdings**:
   - Position 1: 25 @ $150 (remaining from 100)
   - Position 2: 50 @ $160

---

## Portfolio Valuation

### Current Value Calculation

```rust
pub async fn calculate_current_value(
    &self,
    account_id: i32,
) -> Result`Decimal>` {
    // 1. Get holdings
    let holdings = self.calculate_holdings_snapshots(account_id).await?;

    // 2. Fetch current market prices
    let asset_ids: Vec`i32>` = holdings.iter()
        .map(|h| h.asset_id)
        .collect();

    let quotes = self.market_data_service
        .get_latest_quotes(asset_ids)
        .await?;

    // 3. Calculate value per holding
    let mut total_value = Decimal::ZERO;

    for holding in holdings {
        let quote = quotes.get(&holding.asset_id)
            .ok_or(Error::QuoteNotFound)?;

        holding.current_value = holding.quantity * quote.price;

        // Convert to base currency if needed
        let base_currency = self.get_base_currency();
        if quote.currency != base_currency {
            let rate = self.fx_service
                .get_exchange_rate(&quote.currency)
                .await?;
            holding.current_value = holding.current_value * rate;
        }

        total_value += holding.current_value;
    }

    Ok(total_value)
}
```

### TOTAL Account (Aggregation)

Wealthfolio automatically creates a virtual "TOTAL" account that aggregates all
accounts:

```rust
pub async fn get_total_holdings(&self) -> Result`Vec``TotalHolding>`> {
    // 1. Get all accounts
    let accounts = self.account_repo.find_all().await?;

    // 2. Get all holdings from all accounts
    let mut all_holdings: Vec`HoldingSnapshot>` = Vec::new();

    for account in accounts {
        let holdings = self.calculate_holdings_snapshots(account.id).await?;
        all_holdings.extend(holdings);
    }

    // 3. Aggregate by asset
    let mut aggregated: HashMap`i32`, TotalHolding> = HashMap::new();

    for holding in all_holdings {
        let entry = aggregated.entry(holding.asset_id)
            .or_insert_with(|| TotalHolding {
                asset_id: holding.asset_id,
                total_quantity: Decimal::ZERO,
                total_cost: Decimal::ZERO,
                total_value: Decimal::ZERO,
            });

        entry.total_quantity += holding.quantity;
        entry.total_cost += holding.quantity * holding.avg_cost;
        entry.total_value += holding.current_value;
    }

    // 4. Calculate weighted average cost
    let mut total_holdings = Vec::new();

    for (asset_id, mut holding) in aggregated {
        holding.avg_cost = holding.total_cost / holding.total_quantity;
        total_holdings.push(holding);
    }

    Ok(total_holdings)
}
```

---

## Multi-Currency Support

### Currency Conversion

```rust
pub async fn convert_to_base_currency(
    &self,
    amount: Decimal,
    from_currency: &str,
) -> Result`Decimal>` {
    let base_currency = self.get_base_currency();

    // If same currency, no conversion needed
    if from_currency == base_currency {
        return Ok(amount);
    }

    // Get exchange rate
    let rate = self.fx_service
        .get_exchange_rate(from_currency)
        .await?;

    Ok(amount * rate)
}
```

### Example

User's base currency is **USD**:

| Account      | Currency | Holdings Value  | Exchange Rate | Value in USD |
| ------------ | -------- | --------------- | ------------- | ------------ |
| Brokerage    | USD      | $10,000         | 1.0           | $10,000      |
| VN Brokerage | VND      | 250,000,000 VND | 0.00004       | $10,000      |
| **Total**    | -        | -               | -             | **$20,000**  |

---

## Historical Valuations

### Daily Snapshots

Wealthfolio stores daily portfolio valuations:

```rust
pub async fn calculate_daily_valuation(
    &self,
    account_id: i32,
    date: NaiveDate,
) -> Result`AccountValuation>` {
    // 1. Get all activities up to this date
    let activities = self.activity_repo
        .search_activities(
            Some(account_id),
            None,
            None,  // No start date
            Some(date) // Activities up to this date
        )
        .await?;

    // 2. Calculate holdings as of this date
    let holdings = self.calculate_holdings_from_activities(&activities)?;

    // 3. Get market prices as of this date (or nearest)
    let quotes = self.market_data_service
        .get_historical_quotes(&holdings, date, date)
        .await?;

    // 4. Calculate portfolio value
    let value = self.calculate_value_from_holdings(&holdings, &quotes)?;

    // 5. Calculate total contributions
    let contributions = self.calculate_contributions(&activities)?;

    Ok(AccountValuation {
        account_id,
        date,
        value,
        contributions,
    })
}
```

### Database Schema

```sql
CREATE TABLE daily_account_valuation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    date DATE NOT NULL,
    value DECIMAL(20, 8) NOT NULL,
    contributions DECIMAL(20, 8) DEFAULT 0,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_valuation_account_date ON daily_account_valuation(account_id, date);
```

---

## Performance Tracking

### Simple Return

```
Simple Return = (Current Value - Initial Investment) / Initial Investment × 100%
```

### Time-Weighted Return (TWR)

TWR removes the effect of cash flows:

```
TWR = (1 + R1) × (1 + R2) × ... × (1 + Rn) - 1

Where Rn is the return for period n
```

### Money-Weighted Return (MWR/XIRR)

MWR accounts for timing and size of cash flows:

```
Uses XIRR (Extended Internal Rate of Return) algorithm
Considers:
- All BUY and SELL transactions
- All DEPOSITS and WITHDRAWALS
- Current portfolio value
```

See [Performance Analytics](./performance-analytics) for detailed
implementation.

---

## Contribution Tracking

### Total Invested

```rust
pub fn calculate_total_invested(&self, account_id: i32) -> Result`Decimal>` {
    let activities = self.activity_repo
        .search_activities(
            Some(account_id),
            None,
            None,
            None
        )
        .await?;

    let total_invested = activities.iter()
        .filter(|a| matches!(a.activity_type, ActivityType::BUY | ActivityType::DEPOSIT))
        .map(|a| a.quantity * a.unit_price)
        .sum::`Decimal>`();

    Ok(total_invested)
}
```

### Dividend Income

```rust
pub fn calculate_dividend_income(
    &self,
    account_id: i32,
    start_date: Option`NaiveDate>`,
    end_date: Option`NaiveDate>`,
) -> Result`Decimal>` {
    let activities = self.activity_repo
        .search_activities(
            Some(account_id),
            None,
            start_date,
            end_date
        )
        .await?;

    let dividend_income = activities.iter()
        .filter(|a| a.activity_type == ActivityType::DIVIDEND)
        .map(|a| a.quantity * a.unit_price)
        .sum::`Decimal>`();

    Ok(dividend_income)
}
```

---

## Portfolio Aggregation

### Cross-Account View

```typescript
// Frontend: Fetch all holdings from all accounts
export function useTotalHoldings() {
  const { data: accounts } = useAccounts();
  const { data: holdings, isLoading } = useHoldings(undefined); // undefined = all accounts

  // Aggregate by asset across accounts
  const aggregated = useMemo(() => {
    if (!holdings) return [];

    const byAsset = holdings.reduce(
      (acc, h) => {
        const key = h.asset_id;
        acc[key] = acc[key] || {
          asset_id: h.asset_id,
          symbol: h.symbol,
          name: h.name,
          total_quantity: 0,
          total_cost: 0,
          total_value: 0,
        };

        acc[key].total_quantity += h.quantity;
        acc[key].total_cost += h.quantity * h.avg_cost;
        acc[key].total_value += h.current_value;

        return acc;
      },
      {} as Record`number`, any>,
    );

    return Object.values(byAsset).map((h) => ({
      ...h,
      avg_cost: h.total_cost / h.total_quantity,
    }));
  }, [holdings]);

  return { data: aggregated, isLoading };
}
```

---

## Key Features

### 1. Real-Time Updates

Portfolio values update automatically when:

- Market data syncs complete
- Activities are added/modified
- Accounts are created/deleted
- Exchange rates change

### 2. Historical Tracking

All valuations are stored daily, enabling:

- Performance over time charts
- Period returns (YTD, 1Y, 3Y, etc.)
- Growth rate analysis

### 3. Flexible Account Types

Support for various account types:

- Tax-advantaged accounts (401k, IRA)
- Brokerage accounts
- Cash accounts
- Retirement accounts

### 4. Multiple Currencies

Automatic currency conversion:

- Each account can have its own currency
- Base currency setting for display
- Real-time exchange rates

### 5. Contribution Limits

Track against annual limits:

- 401(k) contribution limits
- IRA contribution limits
- Custom limits by account type

---

## Next Steps

- [Performance Analytics](./performance-analytics) - Performance calculation
  details
- [Goals System](./goals-system) - Financial goal tracking
- [Portfolio Calculation Workflow](../workflows/portfolio-calculation) -
  Detailed calculation flow
- [Database Schema](../components/database-schema) - Table structures
