import sys

# Check Python version requirement (3.9+)
if sys.version_info < (3, 9):
    raise RuntimeError(
        f"Python 3.9 or higher is required. Current version: {sys.version_info.major}.{sys.version_info.minor}"
    )

from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from app.models import (
    FundListResponse,
    FundSearchResponse,
    FundQuoteResponse,
    FundHistoryResponse,
    StockSearchResponse,
    StockQuoteResponse,
    StockHistoryResponse,
    IndexQuoteResponse,
    IndexHistoryResponse,
    GoldSearchResponse,
    GoldQuoteResponse,
    GoldHistoryResponse,
    HealthResponse,
    SearchResponse,
    SearchResult
)
from app.clients.fund_client import FundClient
from app.clients.stock_client import StockClient
from app.clients.index_client import IndexClient
from app.clients.gold_client import GoldClient
from app.config import HOST, PORT, CORS_ORIGINS
import logging
from datetime import datetime, timedelta

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="Vietnamese Market Data Service",
    description="Market data provider for Vietnamese assets (stocks, funds, indices) using vnstock",
    version="2.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

fund_client = FundClient()
stock_client = StockClient()
index_client = IndexClient()
gold_client = GoldClient()

@app.get("/health", response_model=HealthResponse)
async def health_check():
    return HealthResponse(
        status="healthy",
        service="vn-market-service",
        version="2.0.0"
    )

@app.get("/funds", response_model=FundListResponse)
async def get_funds_list():
    try:
        funds = fund_client.get_funds_list()
        return FundListResponse(
            funds=[
                {
                    "symbol": f["symbol"],
                    "fund_name": f["fund_name"],
                    "asset_type": f["asset_type"],
                    "data_source": "VN_MARKET"
                }
                for f in funds
            ],
            total=len(funds)
        )
    except Exception as e:
        logger.error(f"Error in get_funds_list: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/funds/search/{symbol}", response_model=FundSearchResponse)
async def search_fund(symbol: str):
    try:
        symbol = symbol.upper()
        fund_info = fund_client.search_fund_by_symbol(symbol)
        
        if not fund_info:
            raise HTTPException(status_code=404, detail=f"Fund {symbol} not found")
        
        return FundSearchResponse(**fund_info)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in search_fund: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/funds/quote/{symbol}", response_model=FundQuoteResponse)
async def get_fund_quote(symbol: str):
    try:
        symbol = symbol.upper()
        quote = fund_client.get_latest_nav(symbol)
        
        if not quote:
            raise HTTPException(status_code=404, detail=f"Quote for {symbol} not found")
        
        return FundQuoteResponse(**quote)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_fund_quote: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/funds/history/{symbol}", response_model=FundHistoryResponse)
async def get_fund_history(
    symbol: str,
    start_date: str = Query(None, description="Start date in YYYY-MM-DD format"),
    end_date: str = Query(None, description="End date in YYYY-MM-DD format")
):
    try:
        symbol = symbol.upper()
        
        if not end_date:
            end_date = datetime.now().strftime("%Y-%m-%d")
        
        if not start_date:
            start_date = (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d")
        
        try:
            datetime.strptime(start_date, "%Y-%m-%d")
            datetime.strptime(end_date, "%Y-%m-%d")
        except ValueError:
            raise HTTPException(
                status_code=400,
                detail="Invalid date format. Use YYYY-MM-DD"
            )
        
        history = fund_client.get_fund_nav_history(symbol, start_date, end_date)
        
        if not history:
            raise HTTPException(
                status_code=404,
                detail=f"No history found for {symbol}"
            )
        
        return FundHistoryResponse(
            symbol=symbol,
            history=history
        )
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_fund_history: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/stocks/search/{symbol}", response_model=StockSearchResponse)
async def search_stock(symbol: str):
    try:
        symbol = symbol.upper()
        stock_info = stock_client.search_stock(symbol)
        
        if not stock_info:
            raise HTTPException(status_code=404, detail=f"Stock {symbol} not found")
        
        return StockSearchResponse(**stock_info)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in search_stock: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/stocks/quote/{symbol}", response_model=StockQuoteResponse)
async def get_stock_quote(symbol: str):
    try:
        symbol = symbol.upper()
        quote = stock_client.get_latest_quote(symbol)
        
        if not quote:
            raise HTTPException(status_code=404, detail=f"Quote for {symbol} not found")
        
        return StockQuoteResponse(**quote)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_stock_quote: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/stocks/history/{symbol}", response_model=StockHistoryResponse)
async def get_stock_history(
    symbol: str,
    start_date: str = Query(None, description="Start date in YYYY-MM-DD format"),
    end_date: str = Query(None, description="End date in YYYY-MM-DD format")
):
    try:
        symbol = symbol.upper()
        
        if not end_date:
            end_date = datetime.now().strftime("%Y-%m-%d")
        
        if not start_date:
            start_date = (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d")
        
        try:
            datetime.strptime(start_date, "%Y-%m-%d")
            datetime.strptime(end_date, "%Y-%m-%d")
        except ValueError:
            raise HTTPException(
                status_code=400,
                detail="Invalid date format. Use YYYY-MM-DD"
            )
        
        history = stock_client.get_stock_history(symbol, start_date, end_date)
        
        if not history:
            raise HTTPException(
                status_code=404,
                detail=f"No history found for {symbol}"
            )
        
        return StockHistoryResponse(
            symbol=symbol,
            history=history
        )
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_stock_history: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/indices/quote/{symbol}", response_model=IndexQuoteResponse)
async def get_index_quote(symbol: str):
    try:
        symbol = symbol.upper()
        quote = index_client.get_latest_quote(symbol)
        
        if not quote:
            raise HTTPException(status_code=404, detail=f"Quote for index {symbol} not found")
        
        return IndexQuoteResponse(**quote)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_index_quote: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/indices/history/{symbol}", response_model=IndexHistoryResponse)
async def get_index_history(
    symbol: str,
    start_date: str = Query(None, description="Start date in YYYY-MM-DD format"),
    end_date: str = Query(None, description="End date in YYYY-MM-DD format")
):
    try:
        symbol = symbol.upper()
        
        if not end_date:
            end_date = datetime.now().strftime("%Y-%m-%d")
        
        if not start_date:
            start_date = (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d")
        
        try:
            datetime.strptime(start_date, "%Y-%m-%d")
            datetime.strptime(end_date, "%Y-%m-%d")
        except ValueError:
            raise HTTPException(
                status_code=400,
                detail="Invalid date format. Use YYYY-MM-DD"
            )
        
        history = index_client.get_index_history(symbol, start_date, end_date)
        
        if not history:
            raise HTTPException(
                status_code=404,
                detail=f"No history found for index {symbol}"
            )
        
        return IndexHistoryResponse(
            symbol=symbol,
            history=history
        )
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_index_history: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/gold/search/VN_GOLD", response_model=GoldSearchResponse)
async def search_gold():
    try:
        gold_info = gold_client.search_gold()
        return GoldSearchResponse(**gold_info)
    except Exception as e:
        logger.error(f"Error in search_gold: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/gold/quote/VN_GOLD", response_model=GoldQuoteResponse)
async def get_gold_quote():
    try:
        quote = gold_client.get_latest_quote()
        
        if not quote:
            raise HTTPException(status_code=404, detail="Gold quote not found")
        
        return GoldQuoteResponse(**quote)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_gold_quote: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/gold/history/VN_GOLD", response_model=GoldHistoryResponse)
async def get_gold_history(
    start_date: str = Query(None, description="Start date in YYYY-MM-DD format"),
    end_date: str = Query(None, description="End date in YYYY-MM-DD format")
):
    try:
        if not end_date:
            end_date = datetime.now().strftime("%Y-%m-%d")
        
        if not start_date:
            start_date = (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d")
        
        try:
            datetime.strptime(start_date, "%Y-%m-%d")
            datetime.strptime(end_date, "%Y-%m-%d")
        except ValueError:
            raise HTTPException(
                status_code=400,
                detail="Invalid date format. Use YYYY-MM-DD"
            )
        
        history = gold_client.get_gold_history(start_date, end_date)
        
        if not history:
            raise HTTPException(
                status_code=404,
                detail="No history found for VN_GOLD"
            )
        
        return GoldHistoryResponse(
            symbol="VN_GOLD",
            history=history
        )
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_gold_history: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/search", response_model=SearchResponse)
async def search_assets(query: str = Query(..., description="Search query for stocks, funds, or indices")):
    try:
        query_upper = query.upper()
        query_lower = query.lower()
        results = []
        
        # Check for VN_GOLD queries - normalize various formats
        gold_patterns = ["vn gold", "vn_gold", "vngold", "gold vn", "vietnam gold", "sjc gold"]
        query_normalized = query_lower.replace("_", " ").replace("-", " ").strip()
        
        # Match "gold" alone or any of the patterns
        if query_normalized == "gold" or any(pattern in query_normalized for pattern in gold_patterns):
            results.append(SearchResult(
                symbol="VN_GOLD",
                name="SJC Gold Price",
                asset_type="COMMODITY",
                exchange="VN",
                currency="VND",
                data_source="VN_MARKET"
            ))
        
        # Known Vietnamese indices
        indices = ["VNINDEX", "VN30", "HNX", "HNX30", "UPCOM"]
        matching_indices = [idx for idx in indices if query_upper in idx]
        
        for idx in matching_indices:
            results.append(SearchResult(
                symbol=idx,
                name=f"Vietnam {idx} Index",
                asset_type="INDEX",
                exchange="HOSE" if idx.startswith("VN") else "HNX",
                currency="VND",
                data_source="VN_MARKET"
            ))
        
        # Search stocks
        try:
            stock_info = stock_client.search_stock(query_upper)
            if stock_info:
                results.append(SearchResult(
                    symbol=stock_info["symbol"],
                    name=stock_info["company_name"],
                    asset_type="STOCK",
                    exchange=stock_info.get("exchange", "HOSE"),
                    currency="VND",
                    data_source="VN_MARKET"
                ))
        except:
            pass
        
        # Search funds
        try:
            funds = fund_client.get_funds_list()
            for fund in funds:
                if query_lower in fund["symbol"].lower() or query_lower in fund["fund_name"].lower():
                    results.append(SearchResult(
                        symbol=fund["symbol"],
                        name=fund["fund_name"],
                        asset_type="FUND",
                        exchange="VN",
                        currency="VND",
                        data_source="VN_MARKET"
                    ))
                    if len(results) >= 20:
                        break
        except:
            pass
        
        return SearchResponse(results=results, total=len(results))
    except Exception as e:
        logger.error(f"Error in search_assets: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/history/{symbol}")
async def get_history(
    symbol: str,
    start_date: str = Query(None, description="Start date in YYYY-MM-DD format"),
    end_date: str = Query(None, description="End date in YYYY-MM-DD format")
):
    try:
        symbol = symbol.upper()
        
        if not end_date:
            end_date = datetime.now().strftime("%Y-%m-%d")
        
        if not start_date:
            start_date = (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d")
        
        # Handle VN_GOLD
        if symbol == "VN_GOLD":
            history = gold_client.get_gold_history(start_date, end_date)
            
            if not history:
                raise HTTPException(
                    status_code=404,
                    detail="No history found for VN_GOLD"
                )
            
            return {
                "symbol": "VN_GOLD",
                "history": history,
                "currency": "VND",
                "data_source": "VN_MARKET"
            }
        
        indices = ["VNINDEX", "VN30", "HNX", "HNX30", "UPCOM"]
        if symbol in indices:
            result = await get_index_history(symbol, start_date, end_date)
            # Ensure response includes all required fields
            return {
                "symbol": result.symbol,
                "history": [item.dict() for item in result.history],
                "currency": result.currency,
                "data_source": result.data_source
            }
        
        fund_symbols = [f["symbol"] for f in fund_client.get_funds_list()]
        if symbol in fund_symbols:
            result = await get_fund_history(symbol, start_date, end_date)
            return {
                "symbol": result.symbol,
                "history": [item.dict() for item in result.history],
                "currency": result.currency,
                "data_source": result.data_source
            }
        
        result = await get_stock_history(symbol, start_date, end_date)
        return {
            "symbol": result.symbol,
            "history": [item.dict() for item in result.history],
            "currency": result.currency,
            "data_source": result.data_source
        }
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_history: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/search/{symbol}")
async def search_asset(symbol: str):
    try:
        symbol_upper = symbol.upper()
        
        # Handle VN_GOLD
        if symbol_upper == "VN_GOLD":
            return {
                "symbol": "VN_GOLD",
                "fund_name": "SJC Gold Price",
                "currency": "VND",
                "data_source": "VN_MARKET"
            }
        
        indices = ["VNINDEX", "VN30", "HNX", "HNX30", "UPCOM"]
        if symbol_upper in indices:
            return {
                "symbol": symbol_upper,
                "fund_name": f"Vietnam {symbol_upper} Index",
                "currency": "VND",
                "data_source": "VN_MARKET"
            }
        
        fund_info = fund_client.search_fund_by_symbol(symbol_upper)
        if fund_info:
            return fund_info
        
        stock_info = stock_client.search_stock(symbol_upper)
        if stock_info:
            return {
                "symbol": stock_info["symbol"],
                "fund_name": stock_info["company_name"],
                "currency": "VND",
                "data_source": "VN_MARKET"
            }
        
        raise HTTPException(status_code=404, detail=f"Asset {symbol} not found")
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in search_asset: {e}")
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    logger.info(f"Starting Vietnamese Market Data Service on {HOST}:{PORT}")
    uvicorn.run(app, host=HOST, port=PORT)
