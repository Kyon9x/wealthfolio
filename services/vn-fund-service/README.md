# Vietnamese Fund Service

FastAPI microservice providing market data for Vietnamese mutual funds using the vnstock library.

## Overview

This service acts as a local API gateway between Wealthfolio and Vietnamese fund data, exposing REST
endpoints that the Rust core can consume.

## Features

- ✅ List all available Vietnamese mutual funds
- ✅ Search fund details by symbol
- ✅ Get latest NAV quotes
- ✅ Fetch historical NAV data with date ranges
- ✅ 24-hour caching for fund listings
- ✅ NAV-to-OHLC mapping for chart compatibility
- ✅ CORS enabled for Tauri integration

## Architecture

```
┌─────────────┐      HTTP      ┌──────────────────┐      vnstock      ┌─────────────┐
│  Wealthfolio │ ◄──────────────► │  FastAPI Service │ ◄────────────────► │   Fund Data │
│  (Rust/Tauri)│                 │   (Python 3.x)   │                   │  (vnstock)  │
└─────────────┘                 └──────────────────┘                   └─────────────┘
```

## Installation

### Prerequisites

- Python 3.8 or higher
- pip3

### Setup

```bash
cd services/vn-fund-service

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

### 1. Health Check

```
GET /health
```

**Response:**

```json
{
  "status": "healthy",
  "service": "vn-fund-service",
  "version": "1.0.0"
}
```

### 2. List All Funds

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
      "data_source": "VN_FUND"
    }
  ],
  "total": 150
}
```

**Notes:**

- Results are cached for 24 hours
- Returns all available Vietnamese mutual funds

### 3. Search Fund by Symbol

```
GET /search/{symbol}
```

**Parameters:**

- `symbol` (path): Fund symbol (e.g., VFMVF1)

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
  "data_source": "VN_FUND"
}
```

**Status Codes:**

- `200`: Success
- `404`: Fund not found
- `500`: Server error

### 4. Get Latest Quote

```
GET /quote/{symbol}
```

**Parameters:**

- `symbol` (path): Fund symbol

**Response:**

```json
{
  "symbol": "VFMVF1",
  "nav": 15234.56,
  "date": "2024-10-26",
  "currency": "VND",
  "data_source": "VN_FUND"
}
```

### 5. Get Historical Data

```
GET /history/{symbol}?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD
```

**Parameters:**

- `symbol` (path): Fund symbol
- `start_date` (query, optional): Start date (default: 1 year ago)
- `end_date` (query, optional): End date (default: today)

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
  "data_source": "VN_FUND"
}
```

**Notes:**

- NAV values are mapped to OHLC for chart compatibility
- Volume is always 0.0 (not applicable to mutual funds)

## Example Usage

### cURL Examples

```bash
# Health check
curl http://127.0.0.1:8765/health

# List all funds
curl http://127.0.0.1:8765/funds

# Search specific fund
curl http://127.0.0.1:8765/search/VFMVF1

# Get latest quote
curl http://127.0.0.1:8765/quote/VFMVF1

# Get 1-year history
curl "http://127.0.0.1:8765/history/VFMVF1?start_date=2023-10-26&end_date=2024-10-26"
```

## Configuration

Edit `app/config.py` to customize:

```python
PORT = 8765                    # Service port
HOST = "127.0.0.1"             # Bind address
CORS_ORIGINS = [               # Allowed origins
    "tauri://localhost",
    "http://localhost:1420"
]
```

## Error Handling

All endpoints return standard HTTP error responses:

```json
{
  "detail": "Error message here"
}
```

Common status codes:

- `400`: Bad Request (invalid parameters)
- `404`: Not Found (symbol doesn't exist)
- `500`: Internal Server Error

## Caching Strategy

- **Fund listings**: Cached for 24 hours
- **Quotes & history**: Fetched fresh on each request
- **Cache fallback**: Returns stale data if vnstock API fails

## Logging

Logs are written to stdout with format:

```
2024-10-26 10:30:00 - app.vnstock_client - INFO - Cached 150 funds
```

## Testing

```bash
# Start service
./start.sh

# In another terminal
curl http://127.0.0.1:8765/health
curl http://127.0.0.1:8765/funds
```

## Integration with Wealthfolio

The Rust provider in `src-core/src/market_data/providers/vn_fund_provider.rs` consumes these
endpoints to:

1. Search and validate Vietnamese fund symbols
2. Fetch asset profiles during symbol search
3. Retrieve historical NAV data for portfolio calculations
4. Get latest quotes for real-time valuations

## Troubleshooting

### Service won't start

- Check Python version: `python3 --version` (need 3.8+)
- Verify dependencies: `pip3 list | grep fastapi`
- Check port availability: `lsof -i :8765`

### vnstock errors

- Update vnstock: `pip3 install --upgrade vnstock`
- Check internet connection
- Verify fund symbol exists

### CORS errors

- Confirm CORS_ORIGINS in config.py
- Check Tauri window origin

## Dependencies

- **fastapi**: Web framework
- **uvicorn**: ASGI server
- **vnstock**: Vietnamese stock market data library
- **pydantic**: Data validation
- **python-dateutil**: Date parsing

## License

Same as Wealthfolio project.
