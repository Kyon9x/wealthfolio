# Vnstock Commands Reference

**Version:** 3.2.0+  
**Generated:** 2025-10-13  
**Official Documentation:** [vnstocks.com](https://vnstocks.com/docs)

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Core Classes & Initialization](#core-classes--initialization)
4. [Listing Commands](#listing-commands)
5. [Quote/Price Data Commands](#quoteprice-data-commands)
6. [Company Information Commands](#company-information-commands)
7. [Financial Reports Commands](#financial-reports-commands)
8. [Trading Data Commands](#trading-data-commands)
9. [Stock Screener Commands](#stock-screener-commands)
10. [Mutual Funds Commands](#mutual-funds-commands)
11. [International Markets Commands](#international-markets-commands)
12. [Miscellaneous Commands](#miscellaneous-commands)
13. [Data Export Commands](#data-export-commands)
14. [Complete Usage Examples](#complete-usage-examples)
15. [Parameters Reference](#parameters-reference)
16. [Data Source Compatibility](#data-source-compatibility)
17. [Tips & Best Practices](#tips--best-practices)

---

## Introduction

**Vnstock** is a comprehensive open-source Python library for Vietnamese stock market analysis and investment automation. It provides:

- **Stock Data**: Real-time quotes, historical prices, intraday tick data
- **Company Information**: Profiles, shareholders, officers, subsidiaries, news, events
- **Financial Reports**: Balance sheets, income statements, cash flows, financial ratios
- **Market Data**: Listings, screeners, trading statistics, foreign/proprietary trading
- **Investment Funds**: Mutual fund data, NAV reports, portfolio holdings
- **International Markets**: FX rates, global indices, cryptocurrencies
- **Miscellaneous**: Exchange rates, gold prices

**Data Sources:**
- **VCI** (Vietstock) - Primary source for Vietnamese stocks
- **TCBS** - Alternative source with rich company data
- **MSN** - International markets (FX, crypto, global indices)
- **Fmarket** - Mutual fund data

---

## Installation

### Stable Release (Recommended)

```bash
pip install -U vnstock
```

### Development Version

```bash
pip install git+https://github.com/thinh-vu/vnstock.git
```

### Verify Installation

```python
import vnstock
print(vnstock.__version__)
```

---

## Core Classes & Initialization

### Main Classes

| Class | Purpose | Supported Sources |
|-------|---------|------------------|
| `Vnstock` | Main facade/aggregator class | VCI, TCBS, MSN |
| `Quote` | Price data (historical, intraday, depth) | VCI, TCBS, MSN |
| `Company` | Company information | VCI, TCBS |
| `Finance` | Financial reports and ratios | VCI, TCBS |
| `Listing` | Symbol listings and classifications | VCI, MSN |
| `Trading` | Trading statistics and market data | VCI, TCBS |
| `Screener` | Stock screening/filtering | TCBS |
| `Fund` | Mutual fund data | Fmarket |

### Import Methods

#### Method 1: Using Vnstock Facade (Recommended)

```python
from vnstock import Vnstock

# Initialize main object
stock = Vnstock().stock(symbol='ACB', source='VCI')

# Access different modules
df_history = stock.quote.history(start='2024-01-01', end='2024-12-31')
df_overview = stock.company.overview()
df_finance = stock.finance.balance_sheet(period='year', lang='en')
```

#### Method 2: Direct Class Import

```python
from vnstock import Quote, Company, Finance, Listing, Trading, Screener

# Initialize individual classes
quote = Quote(symbol='ACB', source='VCI')
company = Company(symbol='ACB', source='VCI')
finance = Finance(symbol='ACB', source='VCI')
listing = Listing(source='VCI')
trading = Trading(source='VCI')
screener = Screener(source='TCBS')
```

#### Method 3: Provider-Specific Import

```python
from vnstock.explorer.vci import Quote, Company, Finance, Trading
from vnstock.explorer.tcbs import Screener

# Use provider-specific implementations
quote = Quote(symbol='ACB')
```

### Initialization Parameters

```python
# Common parameters across classes
Quote(
    symbol='ACB',           # Stock symbol (required for most classes)
    source='VCI',           # Data source: 'VCI', 'TCBS', 'MSN'
    random_agent=False,     # Use random User-Agent headers
    show_log=True          # Display logging information
)

Finance(
    symbol='ACB',
    source='VCI',
    period='quarter',       # 'year' or 'quarter'
    get_all=True,          # Retrieve all available periods
    show_log=True
)
```

### Symbol Conventions

| Asset Type | Symbol Format | Examples |
|------------|---------------|----------|
| Stocks | 3-letter code | ACB, FPT, VNM, VIC, TCB |
| Indices | Index name | VNINDEX, VN30, HNX, UPCOM |
| Futures | Code + Month | VN30F1M, VN30F2M |
| Covered Warrants | Code + CW | CVNM2201, CHPG2202 |
| ETFs | Fund code | E1VFVN30, FUEVFVND |

---

## Listing Commands

Get lists of available symbols, exchanges, industries, and market segments.

### `all_symbols()`

Get all listed stocks across all exchanges.

```python
from vnstock import Listing

listing = Listing(source='VCI')
df = listing.all_symbols()

# Output columns: symbol, organ_short_name, comgroup_code, 
#                 icb_code, exchange
```

**Parameters:**
- `to_df` (bool, default=True): Return as DataFrame

### `symbols_by_exchange()`

Get symbols grouped by exchange/board.

```python
listing = Listing(source='VCI')

# Get all exchanges
df = listing.symbols_by_exchange()

# Filter by language
df = listing.symbols_by_exchange(lang='en')  # 'vi' or 'en'
```

**Output:** DataFrame with columns: exchange, symbol, company_name, icb_code, etc.

### `symbols_by_industries()`

Get symbols grouped by ICB industry classification.

```python
listing = Listing(source='VCI')
df = listing.symbols_by_industries()

# Output: industry_code, industry_name, level, symbols
```

### `industries_icb()`

Get ICB (Industry Classification Benchmark) hierarchy.

```python
listing = Listing(source='VCI')
df = listing.industries_icb()

# Output: icb_code, icb_name, level (1-4), parent_code
```

### `symbols_by_group()`

Get symbols by predefined market groups.

```python
listing = Listing(source='VCI')

# Available groups: VN30, HNX30, UPCOM, CW, FU_INDEX, FU_BOND, BOND
df_vn30 = listing.symbols_by_group(group='VN30')
df_hnx30 = listing.symbols_by_group(group='HNX30')
df_cw = listing.symbols_by_group(group='CW')  # Covered Warrants
```

### `all_future_indices()`

Get all futures index contracts.

```python
listing = Listing(source='VCI')
df = listing.all_future_indices()

# Shortcut for: symbols_by_group(group='FU_INDEX')
```

### `all_government_bonds()`

Get all government bond futures.

```python
listing = Listing(source='VCI')
df = listing.all_government_bonds()

# Shortcut for: symbols_by_group(group='FU_BOND')
```

### `all_covered_warrant()`

Get all covered warrants.

```python
listing = Listing(source='VCI')
df = listing.all_covered_warrant()

# Shortcut for: symbols_by_group(group='CW')
```

### `all_bonds()`

Get all bonds.

```python
listing = Listing(source='VCI')
df = listing.all_bonds()

# Shortcut for: symbols_by_group(group='BOND')
```

---

## Quote/Price Data Commands

Retrieve historical prices, intraday data, and market depth.

### `history()`

Get historical OHLCV (Open, High, Low, Close, Volume) data.

```python
from vnstock import Quote

quote = Quote(symbol='ACB', source='VCI')

# Basic usage
df = quote.history(
    start='2024-01-01',
    end='2024-12-31',
    interval='1D'
)

# Different intervals
df_weekly = quote.history(
    start='2023-01-01',
    end='2024-12-31',
    interval='1W'
)

df_intraday = quote.history(
    start='2024-12-01 09:00:00',
    end='2024-12-01 15:00:00',
    interval='5m'
)

# Change symbol on the fly
df_fpt = quote.history(
    symbol='FPT',
    start='2024-01-01',
    end='2024-12-31'
)
```

**Parameters:**
- `symbol` (str, optional): Override default symbol
- `start` (str): Start date/datetime (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)
- `end` (str): End date/datetime
- `interval` (str): Time interval (see [Interval Reference](#time-intervals))

**Output Columns:** time/date, open, high, low, close, volume

### `intraday()`

Get tick-by-tick intraday trade data.

```python
quote = Quote(symbol='ACB', source='VCI')

# Get latest trades
df = quote.intraday(page_size=1000)

# With pagination
df = quote.intraday(page_size=1000, page=1)

# Different symbol
df = quote.intraday(symbol='FPT', page_size=500)
```

**Parameters:**
- `symbol` (str, optional): Override default symbol
- `page_size` (int, default=100): Number of records per page
- `page` (int, default=1): Page number

**Output Columns:** time, price, volume, side (buy/sell)

### `price_depth()`

Get order book depth (bid/ask levels).

```python
quote = Quote(symbol='ACB', source='VCI')

# Get current order book
df = quote.price_depth()

# Different symbol
df = quote.price_depth(symbol='FPT')
```

**Output:** DataFrame with bid/ask prices and volumes at different levels

---

## Company Information Commands

Access comprehensive company profiles and related information.

### `overview()`

Get company overview and key metrics.

```python
from vnstock import Company

company = Company(symbol='ACB', source='VCI')
df = company.overview()
```

**Output Fields:** company_name, short_name, exchange, industry, 
                    established_year, employees, website, address,
                    phone, charter_capital, shares_outstanding, etc.

### `shareholders()`

Get major shareholders information.

```python
company = Company(symbol='ACB', source='VCI')
df = company.shareholders()
```

**Output Columns:** shareholder_name, ownership_percentage, shares, 
                     shareholder_type, report_date

### `officers()`

Get company officers and management.

```python
company = Company(symbol='ACB', source='VCI')

# All officers
df = company.officers()

# Filter by status
df_working = company.officers(filter_by='working')
df_resigned = company.officers(filter_by='resigned')
df_all = company.officers(filter_by='all')
```

**Parameters:**
- `filter_by` (str): 'working', 'resigned', or 'all'

**Output Columns:** name, position, shares, ownership_percentage, 
                     start_date, status

### `subsidiaries()`

Get information about subsidiaries and affiliated companies.

```python
company = Company(symbol='ACB', source='VCI')

# All subsidiaries
df = company.subsidiaries()

# Filter
df_sub = company.subsidiaries(filter_by='subsidiary')
df_all = company.subsidiaries(filter_by='all')
```

**Parameters:**
- `filter_by` (str): 'subsidiary' or 'all'

**Output Columns:** company_name, ownership_percentage, 
                     charter_capital, industry, address

### `affiliate()`

Get affiliated companies.

```python
company = Company(symbol='ACB', source='VCI')
df = company.affiliate()
```

### `news()`

Get recent company news and announcements.

```python
company = Company(symbol='ACB', source='VCI')
df = company.news()
```

**Output Columns:** date, title, url, source

### `events()`

Get corporate events calendar.

```python
company = Company(symbol='ACB', source='VCI')
df = company.events()
```

**Output Columns:** event_date, event_type, description, 
                     ex_date, record_date, payment_date

---

## Financial Reports Commands

Access financial statements and key ratios.

### `balance_sheet()`

Get balance sheet (statement of financial position).

```python
from vnstock import Finance

finance = Finance(symbol='ACB', source='VCI')

# Annual balance sheet in English
df = finance.balance_sheet(period='year', lang='en', dropna=True)

# Quarterly balance sheet in Vietnamese
df = finance.balance_sheet(period='quarter', lang='vi', dropna=False)
```

**Parameters:**
- `period` (str): 'year' or 'quarter'
- `lang` (str): 'en' or 'vi'
- `dropna` (bool, default=True): Drop rows with all NaN values

**Output:** Multi-column DataFrame with periods as columns and 
            accounts as rows (Assets, Liabilities, Equity)

### `income_statement()`

Get income statement (profit & loss).

```python
finance = Finance(symbol='ACB', source='VCI')

# Annual income statement
df = finance.income_statement(period='year', lang='en', dropna=True)

# Quarterly
df = finance.income_statement(period='quarter', lang='vi')
```

**Parameters:** Same as `balance_sheet()`

**Output:** Revenue, costs, expenses, profit metrics by period

### `cash_flow()`

Get cash flow statement.

```python
finance = Finance(symbol='ACB', source='VCI')

# Annual cash flow
df = finance.cash_flow(period='year', lang='en', dropna=True)

# Quarterly
df = finance.cash_flow(period='quarter', lang='vi')
```

**Parameters:** Same as `balance_sheet()`

**Output:** Operating, investing, and financing cash flows

### `ratio()`

Get financial ratios and key metrics.

```python
finance = Finance(symbol='ACB', source='VCI')

# Get all ratios
df = finance.ratio(period='year', lang='en', dropna=True)

# Quarterly ratios
df = finance.ratio(period='quarter', lang='vi')
```

**Parameters:** Same as `balance_sheet()`

**Output Metrics:** 
- Profitability: ROE, ROA, profit margins
- Liquidity: Current ratio, quick ratio
- Leverage: Debt/equity, debt/assets
- Efficiency: Asset turnover, inventory turnover
- Valuation: P/E, P/B, EPS, BVPS

---

## Trading Data Commands

Access trading statistics, order flow, and market data.

### `trading_stats()`

Get trading statistics and volume analysis.

```python
from vnstock import Trading

trading = Trading(symbol='ACB', source='VCI')

# Get trading stats
df = trading.trading_stats(
    start='2024-01-01',
    end='2024-12-31',
    limit=1000
)
```

**Output:** Daily trading volume, value, average price, etc.

### `side_stats()`

Get bid/ask side statistics.

```python
trading = Trading(symbol='ACB', source='VCI')

# Returns tuple: (bids_df, asks_df)
bids, asks = trading.side_stats(dropna=True)
```

### `price_board()`

Get current price board for multiple symbols.

```python
trading = Trading(source='VCI')

# Get price board for multiple symbols
df = trading.price_board(
    symbols_list=['VCB', 'ACB', 'TCB', 'BID']
)

# Or using list
df = trading.price_board(['FPT', 'VNM', 'VIC'])
```

**Output:** Real-time prices, volume, bid/ask, changes

### `price_history()`

Get price history for trading analysis.

```python
trading = Trading(symbol='ACB', source='VCI')
df = trading.price_history(
    start='2024-01-01',
    end='2024-12-31'
)
```

### `foreign_trade()`

Get foreign investor trading activity.

```python
trading = Trading(symbol='ACB', source='VCI')

df = trading.foreign_trade(
    start='2024-01-01',
    end='2024-12-31'
)
```

**Output Columns:** date, buy_volume, sell_volume, net_volume, 
                     buy_value, sell_value, net_value

### `prop_trade()`

Get proprietary trading activity.

```python
trading = Trading(symbol='ACB', source='VCI')

df = trading.prop_trade(
    start='2024-01-01',
    end='2024-12-31'
)
```

### `insider_deal()`

Get insider trading transactions.

```python
trading = Trading(symbol='ACB', source='VCI')
df = trading.insider_deal()
```

**Output:** Insider name, position, transaction type, shares, price, date

### `order_stats()`

Get order statistics and distribution.

```python
trading = Trading(symbol='ACB', source='VCI')
df = trading.order_stats()
```

---

## Stock Screener Commands

Filter and screen stocks based on criteria.

### `stock()`

Screen stocks with custom filters.

```python
from vnstock import Screener

screener = Screener(source='TCBS')

# Basic screening - all exchanges
df = screener.stock(
    params={"exchangeName": "HOSE,HNX,UPCOM"},
    limit=1700
)

# Filter by specific criteria
df = screener.stock(
    params={
        "exchangeName": "HOSE",
        "marketCap": {"min": 1000, "max": 10000},
        "pe": {"min": 5, "max": 15},
        "roe": {"min": 10}
    },
    limit=100,
    lang='en'
)

# With screener ID (if you have a saved screener)
df = screener.stock(
    params={"exchangeName": "HOSE"},
    id="your_screener_id",
    limit=50
)
```

**Parameters:**
- `params` (dict): Filter criteria dictionary
- `limit` (int, default=50): Maximum results
- `id` (str, optional): Saved screener ID
- `lang` (str, default='vi'): Language ('vi' or 'en')

**Available Filter Fields:**
- `exchangeName`: Exchange codes (HOSE, HNX, UPCOM)
- `marketCap`: Market capitalization range
- `pe`: Price-to-Earnings ratio
- `pb`: Price-to-Book ratio
- `roe`: Return on Equity
- `roa`: Return on Assets
- `eps`: Earnings per Share
- `bvps`: Book Value per Share
- `revenue`: Revenue
- `profit`: Net profit
- `volume`: Trading volume
- `price`: Stock price
- Industry/sector filters (consult TCBS API docs)

**Output:** DataFrame with filtered stocks and their metrics

---

## Mutual Funds Commands

Access mutual fund data, NAV, and portfolio holdings.

### Setup

```python
from vnstock.explorer.fmarket.fund import Fund

fund = Fund(random_agent=False)
```

### `listing()`

Get list of all available mutual funds.

```python
# All funds
df = fund.listing()

# Filter by fund type
df_stock = fund.listing(fund_type='STOCK')
df_bond = fund.listing(fund_type='BOND')
df_balanced = fund.listing(fund_type='BALANCED')
```

**Parameters:**
- `fund_type` (str): '', 'STOCK', 'BOND', 'BALANCED'

**Output Columns:** short_name, full_name, issuer, nav, 
                     nav_change_1m, nav_change_3m, nav_change_6m,
                     nav_change_12m, nav_change_36m, issue_date

### `filter()`

Search for specific funds.

```python
# Search by fund short name
df = fund.filter(symbol='SSISCA')

# Empty string returns all
df = fund.filter(symbol='')
```

### Fund Details

Access detailed fund information using the `details` property.

#### `top_holding()`

Get top portfolio holdings.

```python
df = fund.details.top_holding(symbol='SSISCA')
```

**Output Columns:** stock_code, stock_name, weight, shares, 
                     market_value, report_date

#### `industry_holding()`

Get portfolio breakdown by industry.

```python
df = fund.details.industry_holding(symbol='SSISCA')
```

**Output Columns:** industry_name, weight, market_value

#### `nav_report()`

Get NAV (Net Asset Value) history.

```python
df = fund.details.nav_report(symbol='SSISCA')
```

**Output Columns:** date, nav, change_1d, assets, shares_outstanding

#### `asset_holding()`

Get asset allocation breakdown.

```python
df = fund.details.asset_holding(symbol='SSISCA')    
```

**Output Columns:** asset_type, weight, market_value
                     (e.g., Stocks, Bonds, Cash, etc.)

---

## International Markets Commands

Access FX rates, cryptocurrencies, and global indices.

### Forex (FX)

```python
from vnstock import Vnstock

# Initialize FX
fx = Vnstock().fx(symbol='JPYVND', source='MSN')

# Get historical data
df = fx.quote.history(
    start='2024-01-01',
    end='2024-12-31',
    interval='1D'
)
```

**Available Currency Pairs:**
- Major: EURUSD, GBPUSD, USDJPY, USDCHF, AUDUSD, NZDUSD, USDCAD
- VND: USDVND, EURVND, JPYVND, GBPVND, etc.

### Cryptocurrencies

```python
# Initialize crypto
crypto = Vnstock().crypto(symbol='BTC', source='MSN')

# Get price history
df = crypto.quote.history(
    start='2024-01-01',
    end='2024-12-31',
    interval='1D'
)
```

**Popular Cryptocurrencies:**
- BTC (Bitcoin)
- ETH (Ethereum)
- BNB (Binance Coin)
- XRP (Ripple)
- ADA (Cardano)

### World Indices

```python
# Initialize world index
idx = Vnstock().world_index(symbol='DJI', source='MSN')

# Get historical data
df = idx.quote.history(
    start='2024-01-01',
    end='2024-12-31',
    interval='1D'
)
```

**Popular Global Indices:**
- DJI (Dow Jones)
- SPX (S&P 500)
- IXIC (NASDAQ)
- FTSE (FTSE 100)
- N225 (Nikkei 225)
- HSI (Hang Seng)

---

## Miscellaneous Commands

Additional utility functions for exchange rates and gold prices.

### `vcb_exchange_rate()`

Get exchange rates from Vietcombank.

```python
from vnstock.explorer.misc import vcb_exchange_rate

# Get current exchange rates
df = vcb_exchange_rate()

# Get rates for specific date
df = vcb_exchange_rate(date='2024-12-01')
```

**Output Columns:** currency_code, currency_name, buy_cash, 
                     buy_transfer, sell, date

### `sjc_gold_price()`

Get gold prices from SJC (Saigon Jewelry Company).

```python
from vnstock.explorer.misc import sjc_gold_price

# Current gold prices
df = sjc_gold_price()

# Historical prices (from 2016-01-02)
df = sjc_gold_price(date='2024-12-01')
```

**Output Columns:** name, branch, buy_price, sell_price, date

### `btmc_goldprice()`

Get gold prices from Bao Tin Minh Chau.

```python
from vnstock.explorer.misc import btmc_goldprice

df = btmc_goldprice()
```

**Output Columns:** name, karat, gold_content, buy_price, 
                     sell_price, world_price, time

---

## Data Export Commands

Export data to various formats using pandas.

### Excel Export

```python
from vnstock import Quote

quote = Quote(symbol='ACB', source='VCI')
df = quote.history(start='2024-01-01', end='2024-12-31')

# Export to Excel
df.to_excel('acb_prices.xlsx', index=False)

# With sheet name
df.to_excel('data.xlsx', sheet_name='ACB_Prices', index=False)

# Multiple sheets
with pd.ExcelWriter('report.xlsx') as writer:
    df_prices.to_excel(writer, sheet_name='Prices', index=False)
    df_finance.to_excel(writer, sheet_name='Financials', index=False)
```

### CSV Export

```python
# Basic CSV export
df.to_csv('acb_prices.csv', index=False)

# With UTF-8 BOM (for Excel compatibility)
df.to_csv('acb_prices.csv', index=False, encoding='utf-8-sig')

# Custom separator
df.to_csv('acb_prices.tsv', index=False, sep='\t')
```

### JSON Export

```python
# JSON format
df.to_json('acb_prices.json', orient='records', indent=2)

# JSON lines format
df.to_json('acb_prices.jsonl', orient='records', lines=True)
```

### Parquet Export

```python
# Parquet (efficient columnar format)
df.to_parquet('acb_prices.parquet', index=False)
```

### SQL Database Export

```python
import sqlite3

# Export to SQLite
conn = sqlite3.connect('stocks.db')
df.to_sql('acb_prices', conn, if_exists='replace', index=False)
conn.close()
```

---

## Complete Usage Examples

### Example 1: Basic Stock Analysis Workflow

```python
from vnstock import Vnstock
import pandas as pd

# Initialize
stock = Vnstock().stock(symbol='FPT', source='VCI')

# Get price history
prices = stock.quote.history(
    start='2024-01-01',
    end='2024-12-31',
    interval='1D'
)

# Get company info
company_info = stock.company.overview()

# Get financial ratios
ratios = stock.finance.ratio(period='year', lang='en', dropna=True)

# Export results
prices.to_excel('fpt_analysis.xlsx', sheet_name='Prices', index=False)
ratios.to_excel('fpt_ratios.xlsx', index=False)

print(f"Analyzed {len(prices)} trading days for FPT")
```

### Example 2: Multi-Symbol Comparison

```python
from vnstock import Quote
import pandas as pd

# Define symbols to compare
symbols = ['VCB', 'ACB', 'TCB', 'BID']
quote = Quote(source='VCI')

# Collect data for all symbols
all_data = []
for symbol in symbols:
    df = quote.history(
        symbol=symbol,
        start='2024-01-01',
        end='2024-12-31'
    )
    df['symbol'] = symbol
    all_data.append(df)

# Combine
combined = pd.concat(all_data, ignore_index=True)

# Calculate returns
combined['return'] = combined.groupby('symbol')['close'].pct_change()

# Export
combined.to_csv('bank_stocks_comparison.csv', index=False)
```

### Example 3: Screener to Portfolio

```python
from vnstock import Screener, Quote
import pandas as pd

# Screen for undervalued stocks
screener = Screener(source='TCBS')
filtered = screener.stock(
    params={
        "exchangeName": "HOSE",
        "pe": {"min": 5, "max": 12},
        "roe": {"min": 15},
        "marketCap": {"min": 1000}
    },
    limit=20,
    lang='en'
)

print(f"Found {len(filtered)} stocks matching criteria")

# Get historical prices for top 5
quote = Quote(source='VCI')
top_5 = filtered.head(5)['symbol'].tolist()

portfolio_data = []
for symbol in top_5:
    df = quote.history(
        symbol=symbol,
        start='2024-01-01',
        end='2024-12-31'
    )
    df['symbol'] = symbol
    portfolio_data.append(df)

portfolio = pd.concat(portfolio_data, ignore_index=True)
portfolio.to_csv('screened_portfolio.csv', index=False)
```

### Example 4: Financial Statement Analysis

```python
from vnstock import Finance
import pandas as pd

finance = Finance(symbol='FPT', source='VCI')

# Get all financial statements
bs = finance.balance_sheet(period='year', lang='en', dropna=True)
is_ = finance.income_statement(period='year', lang='en', dropna=True)
cf = finance.cash_flow(period='year', lang='en', dropna=True)
ratios = finance.ratio(period='year', lang='en', dropna=True)

# Export to Excel with multiple sheets
with pd.ExcelWriter('fpt_financials.xlsx') as writer:
    bs.to_excel(writer, sheet_name='Balance Sheet')
    is_.to_excel(writer, sheet_name='Income Statement')
    cf.to_excel(writer, sheet_name='Cash Flow')
    ratios.to_excel(writer, sheet_name='Ratios')

print("Financial statements exported successfully")
```

### Example 5: Intraday Trading Analysis

```python
from vnstock import Quote
import pandas as pd

quote = Quote(symbol='VNM', source='VCI')

# Get intraday ticks
ticks = quote.intraday(page_size=10000)

# Get order book depth
depth = quote.price_depth()

# Analysis
print(f"Total ticks: {len(ticks)}")
print(f"\nPrice Range:")
print(f"High: {ticks['price'].max()}")
print(f"Low: {ticks['price'].min()}")
print(f"Last: {ticks['price'].iloc[-1]}")

print(f"\nOrder Book:")
print(depth)

# Export
ticks.to_csv('vnm_intraday_ticks.csv', index=False)
```

### Example 6: Foreign Trading Activity

```python
from vnstock import Trading
import pandas as pd

trading = Trading(symbol='VIC', source='VCI')

# Get foreign trading data
foreign = trading.foreign_trade(
    start='2024-01-01',
    end='2024-12-31'
)

# Calculate cumulative net foreign flow
foreign['cumulative_net'] = foreign['net_volume'].cumsum()

# Analysis
print(f"Period: {foreign['date'].min()} to {foreign['date'].max()}")
print(f"Total Net Foreign Buy: {foreign['net_volume'].sum():,.0f} shares")
print(f"Average Daily Volume: {foreign['buy_volume'].mean():,.0f} shares")

# Export
foreign.to_excel('vic_foreign_trading.xlsx', index=False)
```

### Example 7: Mutual Fund Portfolio Analysis

```python
from vnstock.explorer.fmarket.fund import Fund
import pandas as pd

fund = Fund()

# Get all stock funds
funds = fund.listing(fund_type='STOCK')
print(f"Found {len(funds)} stock funds")

# Analyze top performer (by 12-month return)
top_fund = funds.nlargest(1, 'nav_change_12m')
symbol = top_fund['short_name'].iloc[0]

print(f"\nAnalyzing top fund: {symbol}")

# Get detailed information
holdings = fund.details.top_holding(symbol=symbol)
industries = fund.details.industry_holding(symbol=symbol)
nav_history = fund.details.nav_report(symbol=symbol)

# Export
with pd.ExcelWriter(f'{symbol}_analysis.xlsx') as writer:
    top_fund.to_excel(writer, sheet_name='Fund Info', index=False)
    holdings.to_excel(writer, sheet_name='Top Holdings', index=False)
    industries.to_excel(writer, sheet_name='Industries', index=False)
    nav_history.to_excel(writer, sheet_name='NAV History', index=False)
```

### Example 8: Multi-Source Data Comparison

```python
from vnstock import Quote
import pandas as pd

symbol = 'ACB'

# Get data from multiple sources
vci_quote = Quote(symbol=symbol, source='VCI')
tcbs_quote = Quote(symbol=symbol, source='TCBS')

vci_data = vci_quote.history(
    start='2024-11-01',
    end='2024-11-30'
)
vci_data['source'] = 'VCI'

tcbs_data = tcbs_quote.history(
    start='2024-11-01',
    end='2024-11-30'
)
tcbs_data['source'] = 'TCBS'

# Combine and compare
comparison = pd.concat([vci_data, tcbs_data], ignore_index=True)
comparison.to_csv('source_comparison.csv', index=False)

# Check for differences
pivot = comparison.pivot_table(
    index='date',
    columns='source',
    values='close'
)
pivot['difference'] = pivot['VCI'] - pivot['TCBS']
print("Price differences between sources:")
print(pivot[pivot['difference'] != 0])
```

### Example 9: Automated Daily Report

```python
from vnstock import Vnstock, Listing
import pandas as pd
from datetime import datetime, timedelta

# Setup
today = datetime.now().strftime('%Y-%m-%d')
yesterday = (datetime.now() - timedelta(days=1)).strftime('%Y-%m-%d')

# Get VN30 symbols
listing = Listing(source='VCI')
vn30 = listing.symbols_by_group(group='VN30')
symbols = vn30['symbol'].tolist()

# Collect data for all VN30 stocks
report_data = []
for symbol in symbols:
    try:
        stock = Vnstock().stock(symbol=symbol, source='VCI')
        
        # Get latest price
        prices = stock.quote.history(
            start=yesterday,
            end=today
        )
        
        if not prices.empty:
            latest = prices.iloc[-1]
            report_data.append({
                'symbol': symbol,
                'date': latest['time'],
                'close': latest['close'],
                'change': latest.get('change', 0),
                'volume': latest['volume']
            })
    except Exception as e:
        print(f"Error processing {symbol}: {e}")
        continue

# Create report
report = pd.DataFrame(report_data)
report = report.sort_values('change', ascending=False)

# Export
filename = f'vn30_daily_report_{today}.xlsx'
report.to_excel(filename, index=False)
print(f"Daily report saved: {filename}")
```

### Example 10: Exchange Rates & Gold Monitoring

```python
from vnstock.explorer.misc import vcb_exchange_rate, sjc_gold_price
import pandas as pd
from datetime import datetime

# Get current data
today = datetime.now().strftime('%Y-%m-%d')

# Exchange rates
fx_rates = vcb_exchange_rate(date=today)

# Gold prices
gold = sjc_gold_price(date=today)

# Summary report
print("=" * 50)
print(f"DAILY MARKET REPORT - {today}")
print("=" * 50)

print("\nTOP EXCHANGE RATES (VND):")
print("-" * 50)
top_fx = fx_rates.head(5)[['currency_code', 'buy_transfer', 'sell']]
print(top_fx.to_string(index=False))

print("\nGOLD PRICES (SJC):")
print("-" * 50)
print(gold[['name', 'buy_price', 'sell_price']].to_string(index=False))

# Export
fx_rates.to_excel(f'fx_rates_{today}.xlsx', index=False)
gold.to_excel(f'gold_prices_{today}.xlsx', index=False)
```

---

## Parameters Reference

### Time Intervals

| Interval | Description | Supported Sources |
|----------|-------------|-------------------|
| `1m` | 1 minute | VCI, TCBS (intraday only) |
| `5m` | 5 minutes | VCI, TCBS (intraday only) |
| `15m` | 15 minutes | VCI, TCBS (intraday only) |
| `30m` | 30 minutes | VCI, TCBS (intraday only) |
| `1H` | 1 hour | VCI, TCBS |
| `1D` or `D` | Daily | VCI, TCBS, MSN |
| `1W` or `W` | Weekly | VCI, TCBS, MSN |
| `1M` or `M` | Monthly | VCI, TCBS, MSN |

**Note:** Intraday intervals (1m-30m) require intraday data subscriptions and may have limitations.

### Period Options

| Period | Description | Use Case |
|--------|-------------|----------|
| `year` | Annual data | Long-term trend analysis |
| `quarter` | Quarterly data | Recent performance tracking |

### Language Options

| Language | Code | Description |
|----------|------|-------------|
| Vietnamese | `vi` | Vietnamese field names and labels |
| English | `en` | English field names and labels |

**Note:** Language parameter affects column names and some descriptive fields.

### Date Formats

| Format | Example | Use Case |
|--------|---------|----------|
| `YYYY-MM-DD` | `2024-01-15` | Daily data |
| `YYYY-MM-DD HH:MM:SS` | `2024-01-15 09:30:00` | Intraday data |

**Timezone:** All times are in Vietnam timezone (UTC+7) unless specified otherwise.

### Exchange Codes

| Code | Exchange Name |
|------|---------------|
| `HOSE` | Ho Chi Minh Stock Exchange |
| `HNX` | Hanoi Stock Exchange |
| `UPCOM` | Unlisted Public Company Market |

### Common Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `symbol` | str | Required* | Stock symbol |
| `source` | str | `'VCI'` | Data source |
| `start` | str | Required | Start date |
| `end` | str | Required | End date |
| `interval` | str | `'1D'` | Time interval |
| `period` | str | `'quarter'` | Report period |
| `lang` | str | `'vi'` | Language |
| `dropna` | bool | `True` | Drop NaN values |
| `page_size` | int | `100` | Results per page |
| `page` | int | `1` | Page number |
| `random_agent` | bool | `False` | Random User-Agent |
| `show_log` | bool | `True` | Display logs |

*Required for most methods except Listing and Screener

---

## Data Source Compatibility

### Feature Support Matrix

| Feature | VCI | TCBS | MSN | Fmarket |
|---------|-----|------|-----|---------|
| **Listing** |
| All Symbols | ✅ | ❌ | ✅ | ❌ |
| By Exchange | ✅ | ❌ | ✅ | ❌ |
| By Industry | ✅ | ❌ | ❌ | ❌ |
| By Group | ✅ | ❌ | ❌ | ❌ |
| **Quote** |
| Historical Daily | ✅ | ✅ | ✅ | ❌ |
| Historical Intraday | ✅ | ✅ | ⚠️ | ❌ |
| Live Intraday | ✅ | ✅ | ❌ | ❌ |
| Price Depth | ✅ | ✅ | ❌ | ❌ |
| **Company** |
| Overview | ✅ | ✅ | ❌ | ❌ |
| Shareholders | ✅ | ✅ | ❌ | ❌ |
| Officers | ✅ | ✅ | ❌ | ❌ |
| Subsidiaries | ✅ | ✅ | ❌ | ❌ |
| News | ✅ | ✅ | ❌ | ❌ |
| Events | ✅ | ✅ | ❌ | ❌ |
| **Financial** |
| Balance Sheet | ✅ | ✅ | ❌ | ❌ |
| Income Statement | ✅ | ✅ | ❌ | ❌ |
| Cash Flow | ✅ | ✅ | ❌ | ❌ |
| Ratios | ✅ | ✅ | ❌ | ❌ |
| **Trading** |
| Trading Stats | ✅ | ✅ | ❌ | ❌ |
| Price Board | ✅ | ✅ | ❌ | ❌ |
| Foreign Trade | ✅ | ✅ | ❌ | ❌ |
| Proprietary Trade | ✅ | ✅ | ❌ | ❌ |
| Insider Deals | ✅ | ✅ | ❌ | ❌ |
| **Screener** |
| Stock Screening | ❌ | ✅ | ❌ | ❌ |
| **International** |
| Forex | ❌ | ❌ | ✅ | ❌ |
| Crypto | ❌ | ❌ | ✅ | ❌ |
| World Indices | ❌ | ❌ | ✅ | ❌ |
| **Funds** |
| Fund Listing | ❌ | ❌ | ❌ | ✅ |
| Fund Details | ❌ | ❌ | ❌ | ✅ |
| NAV History | ❌ | ❌ | ❌ | ✅ |

**Legend:**
- ✅ Fully Supported
- ⚠️ Partially Supported (limited features or intervals)
- ❌ Not Supported

### Source Selection Guide

**Use VCI when:**
- You need comprehensive Vietnamese market data
- Working with listings and classifications
- Need reliable intraday data
- Default/general purpose usage

**Use TCBS when:**
- You need stock screening functionality
- Want richer company information
- Need alternative data verification
- Working with specific TCBS features

**Use MSN when:**
- Accessing international markets (FX, crypto, global indices)
- Need global market data integration
- Working with currency pairs

**Use Fmarket when:**
- Analyzing mutual funds
- Need NAV history and fund holdings
- Working with ETF data

---

## Tips & Best Practices

### 1. Rate Limiting & Polite Usage

```python
import time
from vnstock import Quote

quote = Quote(source='VCI')
symbols = ['ACB', 'VCB', 'TCB', 'BID', 'FPT']

data = []
for symbol in symbols:
    df = quote.history(symbol=symbol, start='2024-01-01', end='2024-12-31')
    data.append(df)
    
    # Be polite - add delay between requests
    time.sleep(0.5)  # 500ms delay
```

### 2. Error Handling

```python
from vnstock import Quote
import pandas as pd

def safe_fetch(symbol, start, end):
    try:
        quote = Quote(symbol=symbol, source='VCI', show_log=False)
        df = quote.history(start=start, end=end)
        return df
    except Exception as e:
        print(f"Error fetching {symbol}: {e}")
        return pd.DataFrame()

# Use with multiple symbols
symbols = ['ACB', 'INVALID', 'FPT']
results = [safe_fetch(s, '2024-01-01', '2024-12-31') for s in symbols]
```

### 3. Caching Data

```python
import os
import pandas as pd
from vnstock import Quote
from datetime import datetime

def get_cached_data(symbol, start, end, cache_dir='cache'):
    """Fetch data with local caching"""
    os.makedirs(cache_dir, exist_ok=True)
    
    cache_file = f"{cache_dir}/{symbol}_{start}_{end}.parquet"
    
    # Check if cached file exists and is recent
    if os.path.exists(cache_file):
        # If file modified today, use cache
        mod_time = datetime.fromtimestamp(os.path.getmtime(cache_file))
        if mod_time.date() == datetime.now().date():
            print(f"Loading {symbol} from cache")
            return pd.read_parquet(cache_file)
    
    # Fetch fresh data
    print(f"Fetching fresh data for {symbol}")
    quote = Quote(symbol=symbol, source='VCI')
    df = quote.history(start=start, end=end)
    
    # Cache it
    df.to_parquet(cache_file)
    return df

# Usage
df = get_cached_data('ACB', '2024-01-01', '2024-12-31')
```

### 4. Handling Missing Data

```python
import pandas as pd
from vnstock import Quote

quote = Quote(symbol='ACB', source='VCI')
df = quote.history(start='2024-01-01', end='2024-12-31')

# Check for missing values
print(f"Missing values:\n{df.isnull().sum()}")

# Forward fill missing values
df_filled = df.fillna(method='ffill')

# Drop rows with any missing values
df_clean = df.dropna()

# Interpolate missing values
df_interpolated = df.interpolate(method='linear')
```

### 5. Working with Multiple Periods

```python
from vnstock import Finance

finance = Finance(symbol='FPT', source='VCI')

# Get both annual and quarterly data
annual = finance.ratio(period='year', lang='en')
quarterly = finance.ratio(period='quarter', lang='en')

# Compare latest quarter vs year
print("Latest Annual ROE:", annual['roe'].iloc[0])
print("Latest Quarterly ROE:", quarterly['roe'].iloc[0])
```

### 6. Efficient Bulk Operations

```python
from vnstock import Listing, Quote
import pandas as pd
from concurrent.futures import ThreadPoolExecutor

listing = Listing(source='VCI')
vn30 = listing.symbols_by_group(group='VN30')
symbols = vn30['symbol'].tolist()

def fetch_symbol(symbol):
    quote = Quote(symbol=symbol, source='VCI', show_log=False)
    return quote.history(start='2024-11-01', end='2024-11-30')

# Parallel fetch (use with caution - respect rate limits)
with ThreadPoolExecutor(max_workers=3) as executor:
    results = list(executor.map(fetch_symbol, symbols))

# Combine results
combined = pd.concat(
    [df.assign(symbol=sym) for df, sym in zip(results, symbols)],
    ignore_index=True
)
```

### 7. Data Validation

```python
from vnstock import Quote
import pandas as pd

quote = Quote(symbol='ACB', source='VCI')
df = quote.history(start='2024-01-01', end='2024-12-31')

# Validate price data
def validate_ohlc(df):
    """Ensure OHLC relationships are valid"""
    invalid = df[
        (df['high'] < df['low']) |
        (df['high'] < df['open']) |
        (df['high'] < df['close']) |
        (df['low'] > df['open']) |
        (df['low'] > df['close'])
    ]
    
    if len(invalid) > 0:
        print(f"Found {len(invalid)} invalid OHLC records")
        print(invalid)
    else:
        print("All OHLC data is valid")
    
    return invalid

invalid_records = validate_ohlc(df)
```

### 8. Date Range Best Practices

```python
from datetime import datetime, timedelta
from vnstock import Quote

# Use relative dates
end_date = datetime.now()
start_date = end_date - timedelta(days=365)  # Last year

quote = Quote(symbol='ACB', source='VCI')
df = quote.history(
    start=start_date.strftime('%Y-%m-%d'),
    end=end_date.strftime('%Y-%m-%d')
)

# Ensure we're not requesting future dates
today = datetime.now().strftime('%Y-%m-%d')
df = quote.history(start='2024-01-01', end=today)
```

### 9. Memory Management for Large Datasets

```python
from vnstock import Quote
import pandas as pd

# Process in chunks for large date ranges
def fetch_by_chunks(symbol, start, end, chunk_months=3):
    """Fetch data in chunks to manage memory"""
    start_dt = pd.to_datetime(start)
    end_dt = pd.to_datetime(end)
    
    chunks = []
    current = start_dt
    
    while current < end_dt:
        chunk_end = min(
            current + pd.DateOffset(months=chunk_months),
            end_dt
        )
        
        quote = Quote(symbol=symbol, source='VCI', show_log=False)
        chunk_df = quote.history(
            start=current.strftime('%Y-%m-%d'),
            end=chunk_end.strftime('%Y-%m-%d')
        )
        chunks.append(chunk_df)
        
        current = chunk_end
    
    return pd.concat(chunks, ignore_index=True)

# Usage for multi-year data
df = fetch_by_chunks('ACB', '2020-01-01', '2024-12-31', chunk_months=6)
```

### 10. Logging and Debugging

```python
from vnstock import Quote
import logging

# Enable detailed logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

# With show_log parameter
quote = Quote(symbol='ACB', source='VCI', show_log=True)
df = quote.history(start='2024-01-01', end='2024-12-31')

# Disable logging for production
quote_quiet = Quote(symbol='ACB', source='VCI', show_log=False)
```

### 11. Data Quality Checks

```python
from vnstock import Quote
import pandas as pd

def data_quality_report(df):
    """Generate data quality report"""
    report = {
        'total_rows': len(df),
        'missing_values': df.isnull().sum().to_dict(),
        'date_range': f"{df['time'].min()} to {df['time'].max()}",
        'duplicates': df.duplicated().sum(),
        'negative_prices': (df['close'] < 0).sum(),
        'zero_volume': (df['volume'] == 0).sum()
    }
    return report

quote = Quote(symbol='ACB', source='VCI')
df = quote.history(start='2024-01-01', end='2024-12-31')

report = data_quality_report(df)
print(pd.Series(report))
```

### 12. Timezone Awareness

```python
from vnstock import Quote
import pandas as pd
import pytz

quote = Quote(symbol='ACB', source='VCI')
df = quote.history(start='2024-01-01', end='2024-12-31')

# Convert to timezone-aware datetime
vietnam_tz = pytz.timezone('Asia/Ho_Chi_Minh')
df['time'] = pd.to_datetime(df['time']).dt.tz_localize(vietnam_tz)

# Convert to UTC for international comparisons
df['time_utc'] = df['time'].dt.tz_convert('UTC')
```

### Important Notes

1. **Disclaimer**: Data is for research purposes only. Not for live trading without verification.

2. **Rate Limits**: Be respectful of API rate limits. Add delays between requests.

3. **Data Accuracy**: Always cross-verify critical data from multiple sources.

4. **Trading Hours**: Vietnamese stock markets trade Monday-Friday, 9:00-15:00 ICT (UTC+7).

5. **Corporate Actions**: Historical prices may or may not be adjusted for splits/dividends depending on source.

6. **License**: Check the vnstock license for usage terms, especially for commercial use.

7. **Updates**: Library is actively maintained. Check GitHub for latest updates.

---

## Additional Resources

- **Official Documentation**: [vnstocks.com](https://vnstocks.com/docs)
- **GitHub Repository**: [github.com/thinh-vu/vnstock](https://github.com/thinh-vu/vnstock)
- **Community**: [Facebook Group](https://www.facebook.com/groups/vnstock.official)
- **Issues & Support**: [GitHub Issues](https://github.com/thinh-vu/vnstock/issues)

---

**Document Version:** 1.0
**Last Updated:** 2025-10-13
**Maintainer:** Vnstock Community

For questions, suggestions, or contributions, please visit the official GitHub repository or join the community forum.
