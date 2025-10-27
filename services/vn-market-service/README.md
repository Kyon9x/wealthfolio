# Vietnamese Market Data Service

FastAPI microservice providing comprehensive market data for Vietnamese assets (stocks, mutual
funds, indices) using the vnstock library.

## Overview

This service acts as a local API gateway between Wealthfolio and Vietnamese market data, exposing
REST endpoints that the Rust core can consume. It supports stocks, mutual funds, and indices.

## Features

- ✅ **Stocks**: Search, quotes, and historical data for Vietnamese stocks
- ✅ **Mutual Funds**: List, search, quotes, and historical NAV data
- ✅ **Indices**: Quotes and historical data for Vietnamese market indices
- ✅ 24-hour caching for fund listings
- ✅ NAV-to-OHLC mapping for chart compatibility
- ✅ CORS enabled for Tauri integration

## Architecture

```
┌─────────────┐      HTTP      ┌──────────────────┐      vnstock      ┌─────────────┐
│  Wealthfolio │ ◄──────────────► │  FastAPI Service │ ◄────────────────► │  Market Data│
│  (Rust/Tauri)│                 │   (Python 3.x)   │                   │  (vnstock)  │
└─────────────┘                 └──────────────────┘                   └─────────────┘
```

## Installation

### Prerequisites

- Python 3.12 or higher (required for vnstock 3.x compatibility)
- pip3

> **Note**: vnstock 3.x requires Python 3.12+. If you're using an older Python version, you'll need
> to upgrade.

### Setup

```bash
cd services/vn-market-service

# Install dependencies
pip3 install -r requirements.txt

# Make start script executable
chmod +x start.sh
```

## Running the Service

### Manual Start

```bash
./start.sh
```

Or:

```bash
python3 -m uvicorn app.main:app --host 127.0.0.1 --port 8765
```

### Auto-start with Tauri

The service is automatically started by Tauri when Wealthfolio launches. See `src-tauri/src/main.rs`
for implementation details.

## API Endpoints

Base URL: `http://127.0.0.1:8765`

### Health Check

```
GET /health
```

**Response:**

```json
{
  "status": "healthy",
  "service": "vn-market-service",
  "version": "2.0.0"
}
```

### Mutual Funds

#### List All Funds

```
GET /funds
```

**Response:**

```json
{
  "funds": [
    {
      "symbol": "VFMVF1",
      "fund_name": "Vietnam Mutual Fund 1",
      "asset_type": "MUTUAL_FUND",
      "data_source": "VN_MARKET"
    }
  ],
  "total": 150
}
```

#### Search Fund by Symbol

```
GET /funds/search/{symbol}
```

**Response:**

```json
{
  "symbol": "VFMVF1",
  "fund_name": "Vietnam Mutual Fund 1",
  "fund_type": "Open-End Fund",
  "management_company": "VFM Fund Management",
  "inception_date": "2020-01-15",
  "nav_per_unit": 15234.56,
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

#### Get Fund Quote

```
GET /funds/quote/{symbol}
```

**Response:**

```json
{
  "symbol": "VFMVF1",
  "nav": 15234.56,
  "date": "2024-10-26",
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

#### Get Fund History

```
GET /funds/history/{symbol}?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD
```

**Response:**

```json
{
  "symbol": "VFMVF1",
  "history": [
    {
      "date": "2024-10-26",
      "nav": 15234.56,
      "open": 15234.56,
      "high": 15234.56,
      "low": 15234.56,
      "close": 15234.56,
      "adjclose": 15234.56,
      "volume": 0.0
    }
  ],
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

### Stocks

#### Search Stock

```
GET /stocks/search/{symbol}
```

**Response:**

```json
{
  "symbol": "VNM",
  "company_name": "Vietnam Dairy Products JSC",
  "exchange": "HOSE",
  "industry": "Food Products",
  "company_type": "Listed Company",
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

#### Get Stock Quote

```
GET /stocks/quote/{symbol}
```

**Response:**

```json
{
  "symbol": "VNM",
  "close": 85000,
  "date": "2024-10-26",
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

#### Get Stock History

```
GET /stocks/history/{symbol}?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD
```

**Response:**

```json
{
  "symbol": "VNM",
  "history": [
    {
      "date": "2024-10-26",
      "open": 84500,
      "high": 85500,
      "low": 84000,
      "close": 85000,
      "adjclose": 85000,
      "volume": 1234567
    }
  ],
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

### Indices

#### Get Index Quote

```
GET /indices/quote/{symbol}
```

**Response:**

```json
{
  "symbol": "VNINDEX",
  "close": 1250.45,
  "date": "2024-10-26",
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

#### Get Index History

```
GET /indices/history/{symbol}?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD
```

**Response:**

```json
{
  "symbol": "VNINDEX",
  "history": [
    {
      "date": "2024-10-26",
      "open": 1248.3,
      "high": 1252.8,
      "low": 1245.2,
      "close": 1250.45,
      "adjclose": 1250.45,
      "volume": 123456789
    }
  ],
  "currency": "VND",
  "data_source": "VN_MARKET"
}
```

## Configuration

Environment variables (see `.env` or `app/config.py`):

```python
VN_MARKET_SERVICE_PORT = 8765
VN_MARKET_SERVICE_HOST = "127.0.0.1"
CORS_ORIGINS = [
    "tauri://localhost",
    "http://localhost:1420"
]
```

## Integration with Wealthfolio

The Rust provider in `src-core/src/market_data/providers/vn_market_provider.rs` consumes these
endpoints to:

1. Search and validate Vietnamese asset symbols (stocks, funds, indices)
2. Fetch asset profiles during symbol search
3. Retrieve historical data for portfolio calculations
4. Get latest quotes for real-time valuations

## Troubleshooting

### Migration to vnstock 3.x

If you're upgrading from vnstock 2.x to 3.x, note the following changes:

- **Python Requirement**: vnstock 3.x requires Python 3.12 or higher
- **API Changes**: The vnstock API has been updated. All clients (stock, fund, index) have been
  migrated to use the new API
- **Virtual Environment**: Recommended to create a fresh virtual environment with Python 3.12+

To migrate:

```bash
# Create new virtual environment with Python 3.12+
python3.12 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install updated dependencies
pip install -r requirements.txt

# Verify vnstock version
pip list | grep vnstock  # Should show 3.2.6 or higher
```

### Service won't start

- Check Python version: `python3 --version` (need 3.10+)
- Verify dependencies: `pip3 list | grep fastapi`
- Verify vnstock version: `pip3 list | grep vnstock` (need 3.2.6+)
- Check port availability: `lsof -i :8765`

### vnstock errors

- Update vnstock: `pip3 install --upgrade vnstock`
- Check internet connection
- Verify symbol exists

## Dependencies

- **fastapi**: Web framework
- **uvicorn**: ASGI server
- **vnstock**: Vietnamese market data library
- **pydantic**: Data validation

## License

Same as Wealthfolio project.
