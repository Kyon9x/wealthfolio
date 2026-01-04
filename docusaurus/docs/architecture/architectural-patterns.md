---
title: Architectural Patterns
sidebar_position: 5
---

# Architectural Patterns

This document describes the key architectural patterns and design principles
used in Wealthfolio.

## Overview

Wealthfolio employs several proven architectural patterns to achieve
maintainability, scalability, and flexibility:

1. **Runtime Abstraction Pattern** - Desktop/Web dual runtime support
2. **Service Layer Architecture** - Clean separation of business logic
3. **Repository Pattern** - Data access abstraction
4. **Adapter Pattern** - Multiple communication protocols
5. **Event-Driven Architecture** - Decoupled communication
6. **Dependency Injection** - Manage component lifecycles
7. **Observer Pattern** - React Query and event listeners
8. **Strategy Pattern** - Pluggable market data providers

---

## 1. Runtime Abstraction Pattern

### Problem

Wealthfolio needs to support both desktop (Tauri) and web (Axum) modes with
identical functionality, but different communication protocols.

### Solution

Use an abstraction layer that detects the runtime environment and routes
requests accordingly.

### Implementation

**TypeScript** (`src/adapters/index.ts`):

```typescript
enum RUN_ENV {
  DESKTOP = "desktop",
  WEB = "web",
  UNSUPPORTED = "unsupported",
}

function getRunEnv(): RUN_ENV {
  if (typeof window !== "undefined" && window.__TAURI__) {
    return RUN_ENV.DESKTOP;
  }
  if (typeof window !== "undefined") {
    return RUN_ENV.WEB;
  }
  return RUN_ENV.UNSUPPORTED;
}
```

**Desktop Adapter** (`src/adapters/tauri.ts`):

```typescript
import { invoke } from "@tauri-apps/api/core";

export async function invokeTauri`T>`(command: string, args?: any): Promise`T>` {
  return invoke`T>`(command, args);
}
```

**Web Adapter** (`src/adapters/web.ts`):

```typescript
export async function invokeWeb`T>`(endpoint: string, body?: any): Promise`T>` {
  const token = localStorage.getItem("access_token");
  const response = await fetch(`/api/v1${endpoint}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  return response.json();
}
```

**Command Wrapper** (`src/commands/account.ts`):

```typescript
import { getRunEnv, invokeTauri, invokeWeb } from "../adapters";

export async function getAccounts(): Promise`Account`[]> {
  const env = getRunEnv();

  if (env === RUN_ENV.DESKTOP) {
    return invokeTauri("get_accounts");
  } else if (env === RUN_ENV.WEB) {
    return invokeWeb("/accounts");
  } else {
    throw new Error("Unsupported runtime environment");
  }
}
```

### Benefits

- **Code Reuse**: Frontend code is 95% identical between desktop and web
- **Type Safety**: All APIs have TypeScript types
- **Easy Switching**: Single flag changes runtime mode
- **Testing**: Mock adapters for unit tests

### Trade-offs

- Slight overhead of runtime detection (negligible)
- Two code paths to maintain (Tauri IPC vs HTTP)
- Web mode requires additional authentication layer

---

## 2. Service Layer Architecture

### Problem

Business logic should be separated from data access and API layers for
maintainability and testability.

### Solution

Implement a service layer that encapsulates business logic and coordinates
between repositories.

### Implementation

**Service Interface** (`src-core/src/services/mod.rs`):

```rust
pub trait AccountService {
    async fn get_all_accounts(&self) -> Result`Vec``Account>`>;
    async fn create_account(&self, account: NewAccount) -> Result`Account>`;
    async fn update_account(&self, id: i32, account: UpdateAccount) -> Result`Account>`;
    async fn delete_account(&self, id: i32) -> Result``()>;
}
```

**Service Implementation** (`src-core/src/services/account.rs`):

```rust
pub struct AccountServiceImpl`R`: AccountRepository> {
    repository: Arc`R>`,
}

impl`R`: AccountRepository> AccountService for AccountServiceImpl`R>` {
    async fn get_all_accounts(&self) -> Result`Vec``Account>`> {
        self.repository.find_all().await
    }

    async fn create_account(&self, account: NewAccount) -> Result`Account>` {
        // Business logic validation
        if account.name.is_empty() {
            return Err(Error::ValidationError("Account name required"));
        }
        self.repository.create(account).await
    }
}
```

**Service Context** (`src-tauri/src/context/mod.rs`):

```rust
pub struct ServiceContext {
    pub account_service: Arc`dyn` AccountService>,
    pub activity_service: Arc`dyn` ActivityService>,
    pub portfolio_service: Arc`dyn` PortfolioService>,
    // ... other services
}

impl ServiceContext {
    pub fn new(db_pool: Pool) -> Result`Self>` {
        let account_repo = Arc::new(AccountRepository::new(db_pool.clone()));
        let activity_repo = Arc::new(ActivityRepository::new(db_pool.clone()));

        let account_service = Arc::new(AccountServiceImpl::new(account_repo));
        let activity_service = Arc::new(ActivityServiceImpl::new(activity_repo));

        Ok(Self {
            account_service,
            activity_service,
            // ...
        })
    }
}
```

**Usage in Tauri Command** (`src-tauri/src/commands/account.rs`):

```rust
#[tauri::command]
pub async fn get_accounts(
    state: tauri::State``'_, Arc`ServiceContext>`>,
) -> Result`Vec``Account>`, String> {
    state.account_service.get_all_accounts()
        .await
        .map_err(|e| e.to_string())
}
```

### Benefits

- **Testability**: Mock services for unit tests
- **Reusability**: Services can be used by Tauri and Axum
- **Maintainability**: Business logic in one place
- **Flexibility**: Easy to swap repository implementations

### Trade-offs

- Additional abstraction layer
- Slight performance overhead (Arc, dynamic dispatch)
- More code to write initially

---

## 3. Repository Pattern

### Problem

Data access logic should be abstracted from business logic to support different
database backends and testing.

### Solution

Use repository pattern to encapsulate data access operations.

### Implementation

**Repository Trait** (`src-core/src/accounts/mod.rs`):

```rust
pub trait AccountRepository: Send + Sync {
    async fn find_all(&self) -> Result`Vec``Account>`>;
    async fn find_by_id(&self, id: i32) -> Result`Option``Account>`>;
    async fn create(&self, account: NewAccount) -> Result`Account>`;
    async fn update(&self, id: i32, account: UpdateAccount) -> Result`Account>`;
    async fn delete(&self, id: i32) -> Result``()>;
}
```

**Repository Implementation** (`src-core/src/accounts/repository.rs`):

```rust
pub struct AccountRepositoryImpl {
    pool: Arc`Pool>`,
}

impl AccountRepository for AccountRepositoryImpl {
    async fn find_all(&self) -> Result`Vec``Account>`> {
        let mut conn = self.pool.get().await?;
        let accounts = accounts::table
            .load::`Account>`(&mut conn)
            .map_err(Error::Database)?;
        Ok(accounts)
    }

    async fn create(&self, account: NewAccount) -> Result`Account>` {
        let mut conn = self.pool.get().await?;
        let account = diesel::insert_into(accounts::table)
            .values(account)
            .returning(Account::as_returning())
            .get_result(&mut conn)
            .map_err(Error::Database)?;
        Ok(account)
    }
}
```

**Diesel Schema** (`src-core/src/accounts/schema.rs`):

```rust
table! {
    accounts (id) {
        id -> Integer,
        name -> Varchar,
        currency -> Varchar,
        account_type -> Varchar,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
```

### Benefits

- **Database Agnostic**: Easy to switch to PostgreSQL, MySQL, etc.
- **Testable**: Mock repositories for unit tests
- **Clean Queries**: SQL logic in one place
- **Type Safety**: Compile-time query validation (Diesel)

### Trade-offs

- Additional abstraction layer
- Limited by ORM capabilities (Diesel features)
- Complex queries can be verbose

---

## 4. Adapter Pattern (Communication)

### Problem

Frontend needs to communicate with backend via different protocols (Tauri IPC vs
HTTP).

### Solution

Use adapter pattern to provide uniform interface for different communication
methods.

### Implementation

**Adapter Interface**:

```typescript
interface BackendAdapter {
  invoke`T>`(command: string, args?: any): Promise`T>`;
}
```

**Tauri Adapter**:

```typescript
class TauriAdapter implements BackendAdapter {
  async invoke`T>`(command: string, args?: any): Promise`T>` {
    return invoke`T>`(command, args);
  }
}
```

**Web Adapter**:

```typescript
class WebAdapter implements BackendAdapter {
  private baseUrl = "/api/v1";

  async invoke`T>`(command: string, args?: any): Promise`T>` {
    const endpoint = this.commandToEndpoint(command);
    const token = localStorage.getItem("access_token");

    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(args),
    });

    return response.json();
  }

  private commandToEndpoint(command: string): string {
    // map Tauri commands to HTTP endpoints
    const mapping: Record`string`, string> = {
      get_accounts: "/accounts",
      create_account: "/accounts",
      update_portfolio: "/portfolio/update",
      // ...
    };
    return this.baseUrl + (mapping[command] || `/${command}`);
  }
}
```

**Adapter Factory**:

```typescript
class AdapterFactory {
  static create(): BackendAdapter {
    const env = getRunEnv();
    return env === RUN_ENV.DESKTOP ? new TauriAdapter() : new WebAdapter();
  }
}

// Usage
const adapter = AdapterFactory.create();
const accounts = await adapter.invoke`Vec``Account>`>("get_accounts");
```

### Benefits

- **Unified Interface**: Frontend code is runtime-agnostic
- **Easy Extension**: Add new adapters (e.g., WebSocket)
- **Testability**: Mock adapters for testing
- **Protocol Independence**: Change backend protocol without frontend changes

### Trade-offs

- Additional abstraction layer
- Performance overhead (minimal)
- Need to maintain command-to-endpoint mappings

---

## 5. Event-Driven Architecture

### Problem

Components need to react to state changes without tight coupling.

### Solution

Use event system for decoupled communication.

### Implementation

**Event Types** (`src-tauri/src/events/mod.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WealthfolioEvent {
    PortfolioUpdateStart,
    PortfolioUpdateComplete,
    PortfolioUpdateError(String),
    MarketSyncStart,
    MarketSyncComplete { count: usize },
    ActivitiesImported { count: usize },
    GoalsUpdated,
}
```

**Event Emission** (`src-tauri/src/lib.rs`):

```rust
use tauri::Manager;

// Emit event to frontend
app.emit_all("portfolio:update-complete", ())?;
```

**Frontend Event Listener** (`src/components/Dashboard.tsx`):

```typescript
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

export function Dashboard() {
  const [isUpdating, setIsUpdating] = useState(false);

  useEffect(() => {
    const unlisten = listen('portfolio:update-start', () => {
      setIsUpdating(true);
    });

    const unlistenComplete = listen('portfolio:update-complete', () => {
      setIsUpdating(false);
      queryClient.invalidateQueries(['holdings']);
    });

    return () => {
      unlisten.then(f => f());
      unlistenComplete.then(f => f());
    };
  }, []);

  return `div>`{isUpdating ? `Spinner` /> : `PortfolioView` />}``/div>;
}
```

**Web Mode (Server-Sent Events)**:

```rust
// Axum SSE endpoint
async fn event_stream(
    State(state): State`Arc``ServiceContext>`>,
) -> Sse`impl` Stream`Item` = Result`Event`, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Subscribe to internal events
    state.subscribe(|event| {
        let _ = tx.send(event);
    });

    Sse::new(
        rx.map(|event| Ok(Event::default().data(serde_json::to_string(&event).unwrap()))),
        keep_alive(),
    )
}
```

### Benefits

- **Loose Coupling**: Components don't know about each other
- **Scalability**: Easy to add new event listeners
- **Real-time Updates**: UI responds to backend changes
- **Async Processing**: Long-running tasks notify progress

### Trade-offs

- Harder to trace execution flow
- Event ordering can be tricky
- Memory leaks if listeners not cleaned up

---

## 6. Dependency Injection

### Problem

Components need dependencies without creating them directly, enabling
testability and flexibility.

### Solution

Use dependency injection container to manage component lifecycles.

### Implementation

**Service Context** (`src-tauri/src/context/mod.rs`):

```rust
pub struct ServiceContext {
    db_pool: Arc`Pool>`,
    repositories: Repositories,
    services: Services,
    caches: Caches,
}

struct Repositories {
    account: Arc`dyn` AccountRepository>,
    activity: Arc`dyn` ActivityRepository>,
    // ...
}

struct Services {
    account: Arc`dyn` AccountService>,
    activity: Arc`dyn` ActivityService>,
    // ...
}

struct Caches {
    quotes: Arc`Cache``String`, Quote>>,
    exchange_rates: Arc`Cache``String`, Decimal>>,
}

impl ServiceContext {
    pub fn new(db_path: &str) -> Result`Self>` {
        // Initialize database pool
        let pool = Arc::new(create_pool(db_path)?);

        // Initialize repositories
        let repositories = Repositories {
            account: Arc::new(AccountRepositoryImpl::new(pool.clone())),
            activity: Arc::new(ActivityRepositoryImpl::new(pool.clone())),
            // ...
        };

        // Initialize caches
        let caches = Caches {
            quotes: Arc::new(Cache::new(Duration::from_secs(900))), // 15 min
            exchange_rates: Arc::new(Cache::new(Duration::from_secs(3600))), // 1 hour
        };

        // Initialize services
        let services = Services {
            account: Arc::new(AccountServiceImpl::new(
                repositories.account.clone(),
            )),
            activity: Arc::new(ActivityServiceImpl::new(
                repositories.activity.clone(),
                repositories.quote.clone(),
                caches.quotes.clone(),
            )),
            // ...
        };

        Ok(Self {
            db_pool: pool,
            repositories,
            services,
            caches,
        })
    }
}
```

**Usage in Tauri** (`src-tauri/src/lib.rs`):

```rust
fn main() {
    let context = Arc::new(ServiceContext::new("wealthfolio.db")?);

    tauri::Builder::default()
        .manage(context)
        .invoke_handler(tauri::generate_handler![
            commands::get_accounts,
            commands::create_account,
            // ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
pub fn get_accounts(
    state: tauri::State``'_, Arc`ServiceContext>`>,
) -> Result`Vec``Account>`, String> {
    state.services.account.get_all_accounts()
        .map_err(|e| e.to_string())
}
```

### Benefits

- **Testability**: Mock services for unit tests
- **Centralized Configuration**: All dependencies in one place
- **Lifecycle Management**: Proper initialization and cleanup
- **Flexibility**: Easy to swap implementations

### Trade-offs

- Additional complexity
- Runtime overhead (Arc, dynamic dispatch)
- Harder to understand data flow

---

## 7. Observer Pattern (React Query)

### Problem

Frontend needs to keep UI in sync with backend data without manual refetching.

### Solution

Use React Query's observer pattern for automatic data synchronization.

### Implementation

**Custom Hook** (`src/hooks/useHoldings.ts`):

```typescript
import { useQuery } from "@tanstack/react-query";
import { getHoldings } from "../commands/portfolio";

export function useHoldings(accountId?: number) {
  return useQuery({
    queryKey: ["holdings", accountId],
    queryFn: () => getHoldings({ accountId }),
    staleTime: 1000 * 60 * 5, // 5 minutes
    enabled: !!accountId,
  });
}
```

**React Component** (`src/components/HoldingsTable.tsx`):

```typescript
export function HoldingsTable({ accountId }: Props) {
  const { data: holdings, isLoading, error } = useHoldings(accountId);

  if (isLoading) return `Spinner` />;
  if (error) return `ErrorDisplay` error={error} />;

  return (
    `table>`
      {holdings?.map(holding => (
        `tr` key={holding.id}>
          `td>`{holding.symbol}``/td>
          `td>`{holding.quantity}``/td>
          `td>`{holding.value}``/td>
        ``/tr>
      ))}
    ``/table>
  );
}
```

**Cache Invalidation**:

```typescript
// After portfolio update
queryClient.invalidateQueries({ queryKey: ["holdings"] });
queryClient.invalidateQueries({ queryKey: ["valuations"] });
```

### Benefits

- **Automatic Caching**: No manual state management
- **Background Refetching**: Data stays fresh
- **Optimistic Updates**: UI updates before backend responds
- **Loading States**: Built-in loading/error states

### Trade-offs

- Additional library dependency
- Complexity for simple use cases
- Cache management overhead

---

## 8. Strategy Pattern (Market Data Providers)

### Problem

Different market data providers (Yahoo Finance, VCI, FMarket, SJC) need to be
used interchangeably.

### Solution

Use strategy pattern with provider-specific implementations.

### Implementation

**Provider Trait** (`src-core/src/market_data/provider.rs`):

```rust
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn fetch_quote(&self, symbol: &str) -> Result`Quote>`;
    async fn fetch_history(&self, symbol: &str, start: DateTime, end: DateTime)
        -> Result`Vec``Quote>`>;
    async fn search_symbols(&self, query: &str) -> Result`Vec``Asset>`>;
}
```

**Yahoo Finance Provider**:

```rust
pub struct YahooFinanceProvider {
    client: reqwest::Client,
}

#[async_trait]
impl MarketDataProvider for YahooFinanceProvider {
    fn name(&self) -> &'static str {
        "Yahoo Finance"
    }

    async fn fetch_quote(&self, symbol: &str) -> Result`Quote>` {
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
        let response = self.client.get(&url).send().await?;
        // Parse response...
    }
}
```

**VCI Provider**:

```rust
pub struct VciProvider {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl MarketDataProvider for VciProvider {
    fn name(&self) -> &'static str {
        "VCI (Vietcap)"
    }

    async fn fetch_quote(&self, symbol: &str) -> Result`Quote>` {
        // VCI-specific implementation
    }
}
```

**Provider Manager** (`src-core/src/market_data/manager.rs`):

```rust
pub struct MarketDataManager {
    providers: HashMap`String`, Box`dyn` MarketDataProvider>>,
}

impl MarketDataManager {
    pub fn new() -> Self {
        let mut providers: HashMap`String`, Box`dyn` MarketDataProvider>> = HashMap::new();

        providers.insert(
            "yahoo".to_string(),
            Box::new(YahooFinanceProvider::new()),
        );
        providers.insert(
            "vci".to_string(),
            Box::new(VciProvider::new()),
        );
        // ... other providers

        Self { providers }
    }

    pub async fn fetch_quote(&self, provider: &str, symbol: &str) -> Result`Quote>` {
        let provider = self.providers.get(provider)
            .ok_or_else(|| Error::ProviderNotFound(provider.to_string()))?;
        provider.fetch_quote(symbol).await
    }
}
```

### Benefits

- **Interchangeable**: Easy to add new providers
- **Testable**: Mock providers for tests
- **Extensible**: Add features without modifying existing code
- **Fallback**: Can use backup providers if one fails

### Trade-offs

- Additional abstraction layer
- Provider-specific quirks need handling
- Interface design can be complex

---

## Cross-Cutting Concerns

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum WealthfolioError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    Network(String),
}
```

### Logging

```rust
use tracing::{info, warn, error};

info!("Portfolio update started");
warn!("Quote not found for symbol: {}", symbol);
error!("Failed to fetch market data: {}", error);
```

### Validation

```typescript
import { z } from "zod";

const AccountSchema = z.object({
  name: z.string().min(1).max(100),
  currency: z.enum(["USD", "VND", "EUR"]),
  accountType: z.enum(["brokerage", "retirement", "cash"]),
});

type Account = z.infer`typeof` AccountSchema>;
```

---

## Next Steps

- [System Context](./system-context) - High-level system view
- [Container Architecture](./container-architecture) - Major components
- [Component Architecture](./component-architecture) - Detailed breakdown
- [Data Flow](./data-flow) - How data flows through the system

For component-specific patterns:

- [Frontend Architecture](../components/frontend) - React patterns
- [Core Services](../components/core-services) - Service patterns
- [Addon System](../components/addon-system) - Addon patterns
