from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from app.models import (
    FundListResponse,
    FundSearchResponse,
    FundQuoteResponse,
    FundHistoryResponse,
    HealthResponse
)
from app.vnstock_client import VnStockClient
from app.config import HOST, PORT, CORS_ORIGINS
import logging
from datetime import datetime, timedelta

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="Vietnamese Fund Service",
    description="Market data provider for Vietnamese mutual funds using vnstock",
    version="1.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

client = VnStockClient()

@app.get("/health", response_model=HealthResponse)
async def health_check():
    return HealthResponse(
        status="healthy",
        service="vn-fund-service",
        version="1.0.0"
    )

@app.get("/funds", response_model=FundListResponse)
async def get_funds_list():
    try:
        funds = client.get_funds_list()
        return FundListResponse(
            funds=[
                {
                    "symbol": f["symbol"],
                    "fund_name": f["fund_name"],
                    "asset_type": f["asset_type"],
                    "data_source": "VN_FUND"
                }
                for f in funds
            ],
            total=len(funds)
        )
    except Exception as e:
        logger.error(f"Error in get_funds_list: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/search/{symbol}", response_model=FundSearchResponse)
async def search_fund(symbol: str):
    try:
        symbol = symbol.upper()
        fund_info = client.search_fund_by_symbol(symbol)
        
        if not fund_info:
            raise HTTPException(status_code=404, detail=f"Fund {symbol} not found")
        
        return FundSearchResponse(**fund_info)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in search_fund: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/quote/{symbol}", response_model=FundQuoteResponse)
async def get_fund_quote(symbol: str):
    try:
        symbol = symbol.upper()
        quote = client.get_latest_nav(symbol)
        
        if not quote:
            raise HTTPException(status_code=404, detail=f"Quote for {symbol} not found")
        
        return FundQuoteResponse(**quote)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in get_fund_quote: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/history/{symbol}", response_model=FundHistoryResponse)
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
        
        history = client.get_fund_nav_history(symbol, start_date, end_date)
        
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

if __name__ == "__main__":
    import uvicorn
    logger.info(f"Starting Vietnamese Fund Service on {HOST}:{PORT}")
    uvicorn.run(app, host=HOST, port=PORT)
