---
title: Data Flow
sidebar_position: 4
---

# Data Flow

This document explains how data flows through Wealthfolio's architecture, from
user interactions to database storage and back to the UI.

## Overview

Wealthfolio's data flow follows a layered architecture with clear separation of
concerns:

1. **User Interaction** → React components and forms
2. **Command Dispatch** → Runtime adapters (Tauri/Web)
3. **Backend Processing** → Services and repositories
4. **Data Persistence** → SQLite database
5. **Cache Layers** → React Query (frontend) + Moka (backend)
6. **UI Updates** → React Query invalidation + Event-driven updates

## Key Data Flows

### 1. Portfolio Update Flow

This is the most complex workflow, involving multiple services and calculations.

```mermaid
sequenceDiagram
    participant User
    participant React as React Component
    participant Cmd as Command Wrapper
    participant Tauri as Tauri Runtime
    participant PS as PortfolioService
    participant SS as SnapshotService
    participant VS as ValuationService
    participant PerfS as PerformanceService
    participant Repo as Repositories
    participant DB as Database
    participant Cache as Moka Cache
    participant Event as Event System

    User->>React: Click "Update Portfolio"
    React->>Cmd: Call updatePortfolio()
    Cmd->>Tauri: invokeTauri('update_portfolio')
    Tauri->>PS: update_portfolio()

    Note over PS: Emit event: portfolio:update-start
    PS->>Event: emit('portfolio:update-start')
    Event->>React: notify listeners
    React->>React: Show loading indicator

    par Calculate Holdings Snapshots
        PS->>SS: calculate_holdings_snapshots()
        SS->>Repo: Get all activities
        Repo->>DB: SELECT * FROM activities
        DB-->>Repo: Activities
        SS->>SS: Apply FIFO algorithm
        Note over SS: Calculate cost basis,`br`/>quantity, avg_price per asset
        SS->>Repo: Save snapshots
        Repo->>DB: INSERT INTO holdings_snapshots
    and Sync Market Data
        PS->>Cache: Check cache (TTL: 15min)
        alt Cache miss
            PS->>PS: Fetch from external APIs
            PS->>Cache: Update cache
        end
    end

    PS->>VS: calculate_valuation_history()
    VS->>Repo: Get holdings snapshots
    VS->>Cache: Get latest quotes
    Note over VS: Calculate: holdings * current_price`br`/>Convert to base currency
    VS->>Repo: Save valuations
    Repo->>DB: INSERT INTO daily_account_valuation

    PS->>PerfS: calculate_performance_summary()
    PerfS->>Repo: Get historical valuations
    Note over PerfS: Calculate: Simple Return,`br`/>CAGR, Money-Weighted Return
    PerfS-->>PS: Performance metrics

    PS-->>Tauri: Success result
    Note over PS: Emit event: portfolio:update-complete
    PS->>Event: emit('portfolio:update-complete')
    Event->>React: notify listeners
    React->>React: Invalidate queries
    React->>React: Re-fetch data
    React-->>User: Show updated portfolio
```

**Steps Explained**:

1. **Trigger**: User clicks "Update Portfolio" button
2. **Event Emission**: Backend emits `portfolio:update-start` event
3. **FIFO Calculation**: Calculate holdings snapshots using First-In-First-Out
   algorithm
4. **Market Data Sync**: Fetch latest prices (use cached if available)
5. **Valuation**: Calculate portfolio value (holdings × current price, currency
   conversion)
6. **Performance**: Calculate performance metrics (returns, CAGR, etc.)
7. **Event Emission**: Backend emits `portfolio:update-complete` event
8. **UI Update**: React Query invalidates cache and re-fetches data

---

### 2. Activity Import Flow (CSV)

Shows how users import activities from broker CSV files.

```mermaid
sequenceDiagram
    participant User
    participant React as React Component
    participant Parser as CSV Parser
    participant Cmd as Command Wrapper
    participant Tauri as Tauri Runtime
    participant AS as ActivityService
    participant Repo as ActivityRepository
    participant AssetRepo as AssetRepository
    participant DB as Database
    participant PS as PortfolioService
    participant Event as Event System

    User->>React: Upload CSV file
    React->>Parser: Parse with PapaParse
    Parser-->>React: Array of raw rows

    User->>React: Configure mapping profile
    Note over React: Map CSV columns → WealthVN fields`br`/>Map symbols → assets
    React->>React: Save mapping profile

    User->>React: Click "Check Import"
    React->>Cmd: checkActivitiesImport(rows, mapping)
    Cmd->>Tauri: invokeTauri('check_activities_import')
    Tauri->>AS: check_import()

    AS->>AS: Validate dates
    AS->>AS: Validate quantities and prices
    AS->>AS: Validate accounts
    AS->>AS: Validate activity types
    AS-->>Tauri: Validation result (errors/warnings)
    Tauri-->>React: Validation report
    React-->>User: Show preview with errors

    User->>React: Click "Import"
    React->>Cmd: importActivities(activities)
    Cmd->>Tauri: invokeTauri('import_activities')
    Tauri->>AS: bulk_create_activities()

    loop For each activity
        AS->>AssetRepo: Find or create asset
        AssetRepo->>DB: SELECT * FROM assets WHERE symbol = ?
        alt Asset not found
            AssetRepo->>DB: INSERT INTO assets
        end
        AS->>Repo: Create activity
        Repo->>DB: INSERT INTO activities
    end

    AS->>PS: Trigger portfolio update
    PS->>PS: update_portfolio()

    Note over PS: Emit event: activities:imported
    PS->>Event: emit('activities:imported')
    Event->>React: notify listeners
    React->>React: Invalidate queries
    React-->>User: Show success message
```

**Key Features**:

- **Mapping Profiles**: Save column mappings for future imports
- **Symbol Resolution**: Automatically map ticker symbols to assets
- **Validation**: Check data before importing
- **Bulk Insert**: Optimized database operations
- **Auto Update**: Trigger portfolio recalculation after import

---

### 3. Market Data Sync Flow

Shows how market data is fetched, cached, and used throughout the app.

```mermaid
flowchart LR
    A[Trigger] --> B{Trigger Type?}

    B -->|Manual| C[User clicks Sync]
    B -->|Automatic| D[Background Task`br`/>Every 15 minutes]
    B -->|On Demand| E[Cache Miss`br`/>Portfolio calculation]

    C --> F[MarketDataService::sync_market_data]
    D --> F
    E --> G[MarketDataService::get_latest_quotes]

    G --> H{Cache Hit?}
    H -->|Yes| I[Return cached data`br`/>TTL: 15 minutes]
    H -->|No| F

    subgraph "Sync All Assets"
        F --> J[Fetch Global Stocks`br`/>Yahoo Finance API]
        F --> K[Fetch VN Stocks`br`/>VCI API]
        F --> L[Fetch VN Funds`br`/>FMarket API]
        F --> M[Fetch Gold Prices`br`/>SJC API]
    end

    J --> N[Update quotes table]
    K --> N
    L --> O[Update vn_historical_records]
    M --> P[Update quotes table]

    N --> Q[Update Moka Cache`br`/>TTL: 15 minutes]
    O --> R[Update Moka Cache`br`/>TTL: 1 day]
    P --> Q

    Q --> S[Emit market:sync-complete event]
    S --> T[Notify frontend]
    T --> U[UI updates`br`/>with new prices]

    I --> U
```

**Caching Strategy**:

| Data Source     | Cache Location              | TTL    | Update Frequency |
| --------------- | --------------------------- | ------ | ---------------- |
| Global quotes   | Moka (backend)              | 15 min | Auto or manual   |
| VN stocks       | Moka + DB                   | 15 min | Auto on startup  |
| VN funds        | Moka + DB                   | 15 min | Auto on startup  |
| Gold prices     | Moka + DB                   | 15 min | Auto on startup  |
| Historical data | vn_historical_records table | N/A    | On sync          |

---

### 4. Request Flow: Desktop vs Web Mode

Shows the differences in request handling between the two runtime modes.

#### Desktop Mode (Tauri)

```mermaid
sequenceDiagram
    participant React as React Component
    participant Hook as use* Hook
    participant Cmd as Command Wrapper
    participant Tauri as Tauri Runtime
    participant Service as Rust Service
    participant Repo as Repository
    participant DB as Database

    React->>Hook: useHoldings(accountId)
    Hook->>Cmd: getHoldings(accountId)
    Cmd->>Cmd: getRunEnv() === 'desktop'
    Cmd->>Tauri: invokeTauri('get_holdings', {accountId})
    Note over Tauri: IPC call (zero network latency)
    Tauri->>Service: portfolio_service.get_holdings(accountId)
    Service->>Repo: repository.find_holdings(accountId)
    Repo->>DB: SELECT * FROM holdings WHERE account_id = ?
    DB-->>Repo: Holdings data
    Repo-->>Service: Holdings
    Service-->>Tauri: Result`Vec``Holding>`>
    Tauri-->>Cmd: Response
    Cmd-->>Hook: Typed data
    Hook->>Hook: React Query cache
    Hook-->>React: Holdings
```

**Characteristics**:

- Zero network latency
- Direct function calls
- Type-safe IPC
- Same process

#### Web Mode (Axum)

```mermaid
sequenceDiagram
    participant React as React Component
    participant Hook as use* Hook
    participant Cmd as Command Wrapper
    participant HTTP as HTTP Request
    participant Axum as Axum Server
    participant Auth as JWT Middleware
    participant Service as Rust Service
    participant Repo as Repository
    participant DB as Database

    React->>Hook: useHoldings(accountId)
    Hook->>Cmd: getHoldings(accountId)
    Cmd->>Cmd: getRunEnv() === 'web'
    Cmd->>HTTP: fetch('/api/v1/holdings?accountId=...')

    Note over HTTP: Network latency (~1-5ms local)

    HTTP->>Axum: GET /api/v1/holdings
    Axum->>Auth: Verify JWT token
    Auth-->>Axum: User context
    Axum->>Service: portfolio_service.get_holdings(accountId)
    Service->>Repo: repository.find_holdings(accountId)
    Repo->>DB: SELECT * FROM holdings WHERE account_id = ?
    DB-->>Repo: Holdings data
    Repo-->>Service: Holdings
    Service-->>Axum: Json(Holdings)
    Axum-->>HTTP: JSON response
    HTTP-->>Cmd: Typed data
    Cmd-->>Hook: Typed data
    Hook->>Hook: React Query cache
    Hook-->>React: Holdings
```

**Characteristics**:

- Network latency (~1-5ms local)
- HTTP/REST protocol
- JWT authentication
- Separate processes

---

### 5. Goal Allocation Flow

Shows how financial goals and account allocations work together.

```mermaid
sequenceDiagram
    participant User
    participant React as React Component
    participant Cmd as Command Wrapper
    participant Tauri as Tauri Runtime
    participant GS as GoalService
    participant Repo as GoalRepository
    participant PS as PortfolioService
    participant DB as Database
    participant Event as Event System

    User->>React: Create new goal
    React->>React: Fill goal form`br`/>Name, Target Amount, Target Date
    React->>Cmd: createGoal(goal)
    Cmd->>Tauri: invokeTauri('create_goal')
    Tauri->>GS: create_goal(goal)
    GS->>Repo: Save goal
    Repo->>DB: INSERT INTO goals
    GS-->>Tauri: New Goal
    Tauri-->>React: Goal with ID
    React-->>User: Show goal created

    User->>React: Allocate accounts to goal
    React->>React: Select accounts`br`/>Set percentage allocations
    React->>Cmd: updateGoalAllocations(goalId, allocations)
    Cmd->>Tauri: invokeTauri('update_goal_allocations')
    Tauri->>GS: update_allocations(goalId, allocations)

    GS->>GS: Validate allocations sum to 100%
    GS->>Repo: Delete old allocations
    Repo->>DB: DELETE FROM goals_allocation WHERE goal_id = ?
    GS->>Repo: Save new allocations
    loop For each allocation
        Repo->>DB: INSERT INTO goals_allocation
    end

    GS->>PS: Calculate goal progress
    PS->>PS: Fetch portfolio value
    PS->>PS: Apply allocation percentages
    PS->>PS: Calculate projected value (CAGR)
    PS-->>GS: Progress data

    GS-->>Tauri: Updated goal with progress
    Note over GS: Emit event: goals:updated
    GS->>Event: emit('goals:updated')
    Event->>React: notify listeners
    React-->>User: Show progress bar
```

**Goal Calculation**:

```
Goal Progress = (Allocated Value / Target Amount) × 100%

Allocated Value = Σ(Account Value × Allocation %)

Projected Value = Current Value × (1 + Target Return)^years

where years = (Target Date - Current Date) / 365
```

---

### 6. Addon Loading and Execution Flow

Shows how addons are loaded and interact with the host application.

```mermaid
sequenceDiagram
    participant User
    participant Runtime as Addon Runtime
    participant Addon as Addon Bundle
    participant Context as AddonsContext
    participant Router as React Router
    participant Query as React Query
    participant Host as Host API

    User->>Runtime: Install addon ZIP
    Runtime->>Runtime: Extract to AppData/addons/
    Runtime->>Runtime: Read manifest.json
    Runtime->>Runtime: Validate manifest

    Runtime->>Addon: Load addon code
    Note over Runtime,Addon: Dynamic import via Blob URL`br`/>(sandboxed execution)

    Addon->>Context: Register addon
    Context->>Context: Create AddonContext
    Note over Context: Inject QueryClient`br`/>Set permissions`br`/>Register routes

    Context->>Router: Add dynamic routes
    Note over Router: Routes added to React Router`br`/>with addon prefix

    Context->>Host: Provide Host API
    Note over Host: Filtered capabilities based`br`/>on permissions

    User->>Runtime: Navigate to addon page
    Router->>Addon: Render addon component
    Addon->>Query: Query portfolio data
    Query->>Query: Use injected QueryClient
    Query->>Query: Call host API
    Host-->>Query: Portfolio data
    Query-->>Addon: Data
    Addon-->>User: Render custom UI

    User->>Runtime: Disable addon
    Context->>Router: Remove addon routes
    Context->>Context: Unload addon
    Router-->>User: Navigate to home
```

**Addon Capabilities** (via Host API):

| Capability        | Permission Required | Example                    |
| ----------------- | ------------------- | -------------------------- |
| Read holdings     | `portfolio:read`    | Fetch user's portfolio     |
| Read activities   | `activities:read`   | Access transaction history |
| Create UI routes  | `routes:create`     | Add custom pages           |
| Add sidebar items | `sidebar:add`       | Add navigation items       |
| Store data        | `storage:write`     | Persist addon settings     |
| HTTP requests     | `network:write`     | Call external APIs         |

---

### 7. Authentication Flow (Web Mode)

Shows how users authenticate in web mode.

```mermaid
sequenceDiagram
    participant User
    participant React as React Component
    participant AuthCtx as AuthContext
    participant HTTP as HTTP Request
    participant Axum as Axum Server
    participant JWT as JWT Handler
    participant DB as Database

    User->>React: Enter credentials
    React->>AuthCtx: login(username, password)
    AuthCtx->>HTTP: POST /api/v1/auth/login
    HTTP->>Axum: Login request

    Axum->>DB: Verify credentials
    DB->>Axum: User found/valid

    Axum->>JWT: Generate JWT token
    Note over JWT: Payload: user_id, exp, iat
    JWT-->>Axum: token string

    Axum-->>HTTP: {access_token, expires_in}
    HTTP-->>AuthCtx: Token
    AuthCtx->>AuthCtx: Store token in localStorage
    AuthCtx->>AuthCtx: Set isAuthenticated = true
    AuthCtx->>AuthCtx: Set Authorization header

    AuthCtx-->>React: Login success
    React-->>User: Redirect to dashboard

    Note over React: Subsequent requests include Authorization header

    React->>AuthCtx: Get headers
    AuthCtx-->>React: {Authorization: Bearer `token>`}

    React->>HTTP: GET /api/v1/holdings
    HTTP->>Axum: Request with token
    Axum->>JWT: Verify token
    JWT->>Axum: User context
    Axum-->>HTTP: Holdings data
    HTTP-->>React: Data
```

**JWT Token Structure**:

```json
{
  "sub": "user_id",
  "exp": 1234567890,
  "iat": 1234567890
}
```

**Authentication State Management**:

- Stored in `localStorage` (persistent)
- Authorization header: `Bearer `token>``
- Automatic token refresh (if configured)
- Logout clears token and state

---

## Data Transformation Examples

### Currency Conversion

When displaying portfolio values in a different currency:

```typescript
// Frontend
const displayValue = (value: number, currency: string) => {
  const rate = exchangeRates[currency]; // Fetched from FxService
  return value * rate; // Convert to base currency
};
```

**Backend**:

```rust
// Rust
pub fn convert_to_base_currency(value: Decimal, from_currency: &str) -> Decimal {
    let rate = self.get_exchange_rate(from_currency);
    value * rate
}
```

### Performance Calculation

**Simple Return**:

```
Simple Return = (Current Value - Initial Investment) / Initial Investment × 100%
```

**CAGR (Compound Annual Growth Rate)**:

```
CAGR = (Current Value / Initial Investment)^(1 / years) - 1
```

**Money-Weighted Return** (XIRR):

```
Uses all cash flows (contributions, withdrawals) and time periods
Implemented in Rust: PerformanceService::calculate_money_weighted_return()
```

---

## Cache Invalidation Patterns

### Automatic Invalidation

| Event                       | Queries Invalidated                     |
| --------------------------- | --------------------------------------- |
| `portfolio:update-complete` | `holdings`, `valuations`, `performance` |
| `activities:imported`       | `holdings`, `activities`, `valuations`  |
| `market:sync-complete`      | `holdings`, `quotes`, `valuations`      |
| `goals:updated`             | `goals`, `goal-progress`                |

### Manual Invalidation

```typescript
// After manual update
queryClient.invalidateQueries({ queryKey: ["holdings"] });
queryClient.invalidateQueries({ queryKey: ["activities"] });
```

---

## Error Handling Flow

```mermaid
sequenceDiagram
    participant React as React Component
    participant Hook as use* Hook
    participant Cmd as Command Wrapper
    participant Backend as Tauri/Axum
    participant Service as Rust Service

    React->>Hook: useHoldings(accountId)
    Hook->>Cmd: getHoldings(accountId)
    Cmd->>Backend: Request

    alt Error Occurs
        Backend--xHook: Error: "Account not found"
        Hook->>Hook: Set error state
        Hook-->>React: {error: "Account not found", data: null}
        React->>React: Show error message
    else Success
        Backend-->>Hook: Holdings data
        Hook-->>React: {error: null, data: Holdings}
        React->>React: Display data
    end
```

**Error Types**:

| Error Level | Example               | Handling                            |
| ----------- | --------------------- | ----------------------------------- |
| Validation  | "Invalid date format" | Show inline error                   |
| Not Found   | "Account not found"   | Show 404 error                      |
| Network     | "Failed to fetch"     | Retry with exponential backoff      |
| Server      | "Internal error"      | Show error message, contact support |

---

## Next Steps

- [Architectural Patterns](./architectural-patterns) - Design patterns used
- [Portfolio Calculation Workflow](../workflows/portfolio-calculation) -
  Detailed FIFO workflow
- [Activity Import Workflow](../workflows/activity-import) - CSV import details
- [Authentication Workflow](../workflows/authentication) - Web mode auth flow
