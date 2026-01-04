---
title: System Context
sidebar_position: 1
---

# System Context

This document provides a high-level view of the Wealthfolio system, its
boundaries, and its interactions with external actors.

## Overview

Wealthfolio is a wealth management application designed for tracking investment
portfolios, financial goals, and market data. It operates as both a desktop
application and a web application, with all user data stored locally on the
user's device.

## System Scope

The Wealthfolio system is responsible for:

- Portfolio tracking across multiple investment accounts
- Performance analytics and historical valuation
- Financial goal management with allocation tracking
- Market data integration (global and Vietnamese markets)
- Multi-currency support with automatic conversion
- Addon system for extensibility
- Local-first data storage (no cloud dependencies)

## External Actors

### Primary Users

- **End Users**: Individuals or families managing their investment portfolios
  and financial goals. They interact with Wealthfolio through either:
  - Desktop application (Windows, macOS, Linux)
  - Web browser (Chrome, Firefox, Safari, Edge)

### External Systems

- **Market Data Providers**:
  - Yahoo Finance API - Global market data (stocks, ETFs, mutual funds)
  - VCI (Vietcap) - Vietnamese stock market data
  - FMarket - Vietnamese mutual fund data
  - SJC - Vietnamese gold prices

- **Operating System**:
  - File system - For storing SQLite database and user data
  - Keyring/Secret Store - For secure storage of API keys and credentials
  - Desktop integration - For native window management and notifications

- **Addon Registry** (optional):
  - Community-hosted addon marketplace
  - Update checking for installed addons

## System Boundary Diagram

```mermaid
flowchart TB
    subgraph "Wealthfolio System"
        direction TB
        U[End User]
        W[Wealthfolio]
    end

    subgraph "External Systems"
        YF[Yahoo Finance API]
        VC[VCI (Vietcap)]
        FM[FMarket]
        SJ[SJC]
        AR[Addon Registry]
    end

    U -->|Uses| W
    W -->|HTTPS API| YF
    W -->|HTTPS API| VC
    W -->|HTTPS API| FM
    W -->|HTTPS API| SJ
    W -->|HTTPS| AR
```

## System Characteristics

### Local-First Architecture

All user data is stored locally on the user's device:

- Investment accounts and holdings
- Transaction activities
- Market data cache
- User settings and preferences
- Installed addons

**Benefits**:

- No cloud dependency
- Complete data ownership
- Offline functionality
- Privacy and security

### Dual Runtime Support

Wealthfolio operates in two modes with identical functionality:

1. **Desktop Mode** (Tauri):
   - Native application with OS integration
   - Direct Rust backend communication
   - Local SQLite database
   - Background data synchronization

2. **Web Mode** (Axum):
   - Browser-based application
   - HTTP API communication
   - Server-side SQLite database
   - Real-time updates via Server-Sent Events

### Extensibility

Wealthfolio supports a powerful addon system that allows:

- Custom UI components and pages
- Additional data integrations
- Custom analytics and reports
- Third-party service integrations
- Business logic extensions

Addons are developed in TypeScript using the Wealthfolio Addon SDK.

## Data Ownership & Privacy

### Data Stored Locally

All sensitive user data remains on the user's device:

- Account credentials (encrypted via OS keyring)
- Investment positions and transactions
- Performance history
- Goals and allocations
- User settings

### No Cloud Sync (by default)

Wealthfolio does not sync data to cloud services by default. Users maintain full
control over their data.

### Optional Cloud Integration

Future releases may include optional cloud backup/sync, but this will be opt-in
with transparent encryption.

## Use Cases

### Primary Use Cases

1. **Portfolio Management**
   - Track multiple investment accounts
   - Monitor holdings across asset classes
   - View portfolio performance over time
   - Calculate returns and growth

2. **Activity Tracking**
   - Record buy/sell transactions
   - Import CSV files from brokers
   - Track dividends and interest
   - Monitor contributions and withdrawals

3. **Goal Planning**
   - Set financial goals with target amounts
   - Allocate accounts to goals
   - Track progress toward targets
   - Project future values

4. **Market Analysis**
   - Access real-time market data
   - View historical price charts
   - Compare performance across assets
   - Analyze market trends

5. **Addon Development**
   - Extend application functionality
   - Create custom visualizations
   - Integrate external APIs
   - Share with community

## Technical Requirements

### Desktop Mode

- **Platform**: Windows 10+, macOS 11+, or Linux (modern distributions)
- **Dependencies**: Installed via native installer or portable executable
- **Storage**: ~200MB application + user data
- **Network**: Optional (for market data updates)

### Web Mode

- **Browser**: Chrome 90+, Firefox 88+, Safari 14+, Edge 90+
- **Network**: Required (connects to Axum server)
- **Server**: Single-node deployment on Linux/Windows/macOS

## System Metrics

As of current version:

- **Codebase Size**:
  - Rust backend: ~213,000 LOC
  - TypeScript frontend: ~75,000 LOC
  - Total files: ~735 source files

- **Dependencies**:
  - Core Rust crates: 50+
  - NPM packages: 100+
  - External APIs: 4 market data providers

- **Capabilities**:
  - Supports unlimited accounts
  - Tracks unlimited activities
  - Manages unlimited goals
  - Extensible via addon system

## Next Steps

To understand the internal architecture, see:

- [Container Architecture](./container-architecture) - C4 Level 2 breakdown
- [Component Architecture](./component-architecture) - Detailed component
  relationships
- [Architectural Patterns](./architectural-patterns) - Key design patterns used

For development guidance:

- [Development Overview](../development/overview) - Setup and workflow
- [API Reference](../api/overview) - Complete API documentation
