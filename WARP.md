# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Essential Commands

### Development
```bash
# Start development server (React + Tauri)
pnpm tauri dev

# Frontend only development
pnpm dev

# Build for production
pnpm tauri build

# Build frontend only
pnpm build
```

### Testing
```bash
# Run all tests
pnpm test

# Watch mode
pnpm test:watch

# Test with UI
pnpm test:ui

# Coverage report
pnpm test:coverage

# Run single test file
pnpm test src/lib/portfolio-helper.test.ts

# Run single test by name
pnpm test -- -t "specific test name"
```

### Rust Backend
```bash
# Build Rust backend
cd src-tauri && cargo build

# Run Rust tests
cd src-tauri && cargo test

# Run single Rust test
cd src-tauri && cargo test test_name_substring

# Lint Rust code
cd src-tauri && cargo clippy -- -D warnings

# Format Rust code
cd src-tauri && cargo fmt --all
```

### Code Quality
```bash
# TypeScript type checking
pnpm tsc

# Lint frontend code
pnpm lint

# Format code with Prettier
pnpm format
```

### Addon Development
```bash
# Create new addon
npx @wealthfolio/addon-dev-tools create my-addon

# Start addon development server
pnpm addon:dev

# Create addon via workspace tools
pnpm addon:create
```

## Architecture

Wealthfolio is a **Tauri desktop application** with:

- **Frontend**: React 18 + Vite + TanStack Query + React Router
- **Backend**: Rust with Tauri framework
- **Database**: SQLite with Diesel ORM
- **UI**: Tailwind CSS + Radix UI (shadcn/ui)
- **Package Management**: pnpm with workspace support

### Tauri IPC Commands

The frontend communicates with Rust backend via Tauri commands organized by domain:

- **Portfolio**: `get_holdings`, `update_portfolio`, `calculate_performance_*`
- **Accounts**: `get_accounts`, `create_account`, `update_account`, `delete_account`
- **Activities**: `search_activities`, `import_activities`, `create_activity`
- **Market Data**: `sync_market_data`, `search_symbol`, `get_quote_history`
- **Settings**: `get_settings`, `update_settings`, `get_latest_exchange_rates`
- **Goals**: `create_goal`, `update_goal`, `get_goals`, `update_goal_allocations`
- **Addons**: `install_addon_zip`, `list_installed_addons`, `toggle_addon`
- **Utilities**: `backup_database`, `restore_database`

Commands are defined in `src-tauri/src/commands/` and invoked from React via `@tauri-apps/api`.

### Addon System

Wealthfolio features a powerful **TypeScript addon system**:

- **SDK**: `@wealthfolio/addon-sdk` provides type-safe APIs
- **Dev Tools**: `@wealthfolio/addon-dev-tools` for scaffolding and hot reload
- **UI Library**: `@wealthfolio/ui` for consistent styling
- **Runtime**: Dynamic route injection, event system, secure storage

Addons can:
- Add custom pages and navigation items
- Access portfolio, account, and market data
- Listen to real-time events
- Store secrets securely via permissions system

## Development Patterns

### Commit Convention
Use **Conventional Commits**: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`

### Frontend Guidelines
- **TypeScript**: Strict mode enabled, prefer interfaces over types
- **Components**: Functional components with hooks, avoid classes
- **Styling**: Tailwind CSS with design tokens, CSS variables for theming
- **File Structure**: Co-locate components by feature, use barrel exports
- **Path Aliases**: `@/` for `src/`, `@wealthfolio/ui` for UI components

### Rust Guidelines
- **Error Handling**: Use `Result<T, E>` and `?` operator, custom error types with `thiserror`
- **Async**: Tokio runtime with proper `await` points
- **Database**: Diesel ORM with migrations, connection pooling via r2d2
- **Code Style**: Snake_case for functions/variables, PascalCase for types

### Naming Preference
- Use **"portfolio"** terminology over "assets" (e.g., `user-portfolio-details` vs `Asset Performance Detail System`)
- This avoids overlap with the user-holdings service focused on individual assets

## Directory Structure

```
wealthfolio/
├── src/                          # React frontend
│   ├── components/               # Reusable UI components
│   ├── pages/                    # Route components by feature
│   │   ├── dashboard/           # Portfolio overview
│   │   ├── holdings/            # Asset positions
│   │   ├── activity/            # Transaction management
│   │   ├── settings/            # App configuration
│   │   └── onboarding/          # New user flow
│   ├── hooks/                    # Custom React hooks
│   ├── lib/                      # Utilities and schemas
│   ├── commands/                 # Tauri IPC wrappers
│   └── addons/                   # Addon system core
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/             # IPC command handlers
│   │   ├── addons/              # Addon management
│   │   └── context/             # Service context
│   └── Cargo.toml               # Rust dependencies
├── src-core/                     # Core business logic (Rust)
│   └── src/                     # Domain services and models
├── packages/                     # Workspace packages
│   ├── addon-sdk/               # TypeScript SDK for addons
│   ├── addon-dev-tools/         # CLI and dev server
│   └── ui/                      # Shared UI components
├── addons/                       # Example addons
│   ├── goal-progress-tracker/
│   └── investment-fees-tracker/
└── docs/                         # Documentation
    ├── addons/                  # Addon development guides
    └── activities/              # Activity types reference
```

## Database & Storage

- **Engine**: SQLite with file-based storage
- **ORM**: Diesel with migration support
- **Location**: Configurable via `DATABASE_URL` env var (default: `../db/wealthfolio.db`)
- **Migrations**: Located in `src-core/migrations/`, run automatically on startup
- **Backup/Restore**: Available via Tauri commands and UI

### Environment Setup
```bash
# Copy environment template
cp .env.example .env

# Configure database path (relative to src-tauri/)
DATABASE_URL=../db/wealthfolio.db
```

## Development Environment

### Prerequisites
- **Node.js**: Version specified in `package.json` engines
- **pnpm**: Fast package manager (required for workspaces)
- **Rust**: Latest stable toolchain
- **Tauri Prerequisites**: Platform-specific dependencies (see [Tauri docs](https://tauri.app/v1/guides/getting-started/prerequisites))

### Setup
```bash
# Install dependencies
pnpm install

# Set up environment
cp .env.example .env

# Start development
pnpm tauri dev
```

### DevContainer Support
Available with VS Code + Remote-Containers extension:
- Pre-configured Rust and Node.js environment
- X11 virtual display with VNC access (port 5900)
- GPU support and persistent caches

## Testing

- **Framework**: Vitest for frontend unit/integration tests
- **Setup**: `src/test/setup.ts` configures jsdom environment
- **Rust**: Standard `cargo test` with unit and integration tests
- **Coverage**: Available via `pnpm test:coverage`

### Running Tests
```bash
# All frontend tests
pnpm test

# Specific test file
pnpm test src/lib/portfolio-helper.test.ts

# Specific test case
pnpm test -- -t "should calculate portfolio performance"

# Rust tests
cd src-tauri && cargo test

# Single Rust test
cd src-tauri && cargo test portfolio_calculation
```

## Key Features

- **Local-First**: All data stored locally, no cloud dependencies
- **Multi-Currency**: Exchange rate management and conversion
- **Goal Tracking**: Financial planning with allocation management
- **Activity Import**: CSV import with flexible mapping
- **Performance Analytics**: Historical analysis and benchmarking
- **Extensible**: Rich addon system for custom functionality