# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Wealthfolio is a **beautiful and boring desktop investment tracker** with local data storage, built using Tauri (Rust backend) and React (frontend). This branch `fix-vn-prefix` is specifically focused on fixing Vietnamese market data prefix functionality.

## Essential Commands

### Setup and Installation
```bash
# Install dependencies
pnpm install

# Copy environment configuration
cp .env.example .env
```

### Development Modes
```bash
# Desktop app development (default)
pnpm tauri dev

# Web mode (browser + REST API)
pnpm run dev:web

# Server only (Rust backend)
cargo run --manifest-path src-server/Cargo.toml
```

### Building and Testing
```bash
# Build for production
pnpm tauri build

# Type checking
pnpm type-check

# Linting and formatting
pnpm lint
pnpm format

# Testing
pnpm test
pnpm test:coverage
```

### Addon Development
```bash
# Create new addon
pnpm addon:create my-addon

# Start addon development server
pnpm addon:dev

# Publish addon packages
pnpm publish:sdk
pnpm publish:ui
```

## Architecture Overview

### Technology Stack
- **Frontend**: React 19.1.1 + TypeScript + Vite + Tailwind CSS + TanStack Query
- **Backend**: Rust + Tauri + SQLite + Diesel ORM + Axum web server
- **Monorepo**: pnpm workspaces with Turborepo for builds

### Key Components
- **`src/`**: Main React application with pages, components, and addons system
- **`src-core/`**: Rust core business logic and market data providers
- **`src-tauri/`**: Tauri desktop app code
- **`src-server/`**: Rust web server (Axum)
- **`packages/`**: Shared packages (addon-sdk, addon-dev-tools, ui)
- **`addons/`**: Example addons

### Current Development Focus
The `fix-vn-prefix` branch is working on Vietnamese market data integration:
- Environment variables: `VN_MARKET_SERVICE_PORT`, `VN_MARKET_SERVICE_URL`
- Modified file: `src-core/src/market_data/providers/provider_registry.rs`
- Vietnamese market symbol detection and handling

## Development Guidelines

### Frontend Rules (from .cursor/rules/frontend-rules.mdc)
- Write concise, technical TypeScript code
- Use functional components with TypeScript interfaces
- Prefer interfaces over types, avoid enums
- Use descriptive variable names with auxiliary verbs (e.g., `isLoading`, `hasError`)
- Use Tailwind CSS for styling
- Focus on immutable data structures and performance optimization

### Rust Rules (from .cursor/rules/rust-rules.mdc)
- Write idiomatic Rust with async programming patterns
- Use expressive variable names (snake_case for functions, PascalCase for types)
- Embrace Rust's Result and Option types for error handling
- Implement custom error types using `thiserror`
- Prioritize modularity and clean code organization
- Use `?` operator to propagate errors in async functions

### Code Structure
1. Exported component
2. Subcomponents
3. Helpers
4. Static content
5. Types

## Market Data Provider System

The application uses a pluggable provider architecture:
- **Yahoo Finance**: Default provider for international markets
- **Alpha Vantage**: Alternative data provider
- **VN Market**: Vietnamese market provider (current focus)
- Priority-based provider selection with fallback mechanisms

## Addon System

Wealthfolio features an extensible addon architecture:
- TypeScript SDK with full type safety and hot reload
- Permission-based security system with user consent
- Dynamic route injection for addon pages
- Full access to portfolio, accounts, and market data
- Secure API key management through OS keyring

## Environment Configuration

Key environment variables for this branch:
```env
DATABASE_URL=../db/app.db
VN_MARKET_SERVICE_PORT=8765
VN_MARKET_SERVICE_URL=http://127.0.0.1:8765
```

## Entry Points

### Main Application
- **React app**: `src/main.tsx` (app initialization)
- **Tauri app**: `src-tauri/src/lib.rs` (desktop entry point)
- **Routing**: `src/routes.tsx` (application routing)

### Configuration
- **Vite**: `vite.config.ts` (build configuration with proxy settings)
- **TypeScript**: `tsconfig.json` (path mappings and strict typing)
- **Tailwind**: `tailwind.config.js` (styling configuration)

## Testing and Quality Assurance

- **Frontend**: Vitest with React Testing Library
- **Type checking**: TypeScript strict mode
- **Linting**: ESLint with React and TypeScript rules
- **Formatting**: Prettier with Tailwind plugin

## Security and Privacy

- **Local data storage**: SQLite database with no cloud dependencies
- **API key management**: OS keyring integration (`keyring` crate)
- **Addon permissions**: Comprehensive permission system with user consent
- **No cloud dependencies**: Truly offline-first approach

## Database

- **ORM**: Diesel with SQLite
- **Location**: Configurable via `DATABASE_URL` (defaults to `../db/app.db`)
- **Schema**: Centralized management in `src-core/src/schema.rs`