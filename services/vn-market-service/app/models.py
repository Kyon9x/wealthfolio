from pydantic import BaseModel
from typing import List, Optional

class FundBasicInfo(BaseModel):
    symbol: str
    fund_name: str
    asset_type: str
    data_source: str = "VN_MARKET"

class FundListResponse(BaseModel):
    funds: List[FundBasicInfo]
    total: int

class FundSearchResponse(BaseModel):
    symbol: str
    fund_name: str
    fund_type: Optional[str] = None
    management_company: Optional[str] = None
    inception_date: Optional[str] = None
    nav_per_unit: Optional[float] = None
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class FundQuoteResponse(BaseModel):
    symbol: str
    nav: float
    date: str
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class FundHistoryItem(BaseModel):
    date: str
    nav: float
    open: float
    high: float
    low: float
    close: float
    adjclose: float
    volume: float = 0.0

class FundHistoryResponse(BaseModel):
    symbol: str
    history: List[FundHistoryItem]
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class StockSearchResponse(BaseModel):
    symbol: str
    company_name: str
    exchange: str
    industry: Optional[str] = None
    company_type: Optional[str] = None
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class StockQuoteResponse(BaseModel):
    symbol: str
    close: float
    date: str
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class StockHistoryItem(BaseModel):
    date: str
    nav: float
    open: float
    high: float
    low: float
    close: float
    adjclose: float
    volume: float

class StockHistoryResponse(BaseModel):
    symbol: str
    history: List[StockHistoryItem]
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class IndexQuoteResponse(BaseModel):
    symbol: str
    close: float
    date: str
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class IndexHistoryItem(BaseModel):
    date: str
    nav: float
    open: float
    high: float
    low: float
    close: float
    adjclose: float
    volume: float

class IndexHistoryResponse(BaseModel):
    symbol: str
    history: List[IndexHistoryItem]
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class HealthResponse(BaseModel):
    status: str
    service: str
    version: str

class SearchResult(BaseModel):
    symbol: str
    name: str
    asset_type: str
    exchange: str
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class SearchResponse(BaseModel):
    results: List[SearchResult]
    total: int

class GoldSearchResponse(BaseModel):
    symbol: str
    name: str
    asset_type: str
    exchange: str
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class GoldQuoteResponse(BaseModel):
    symbol: str
    close: float
    date: str
    buy_price: Optional[float] = None
    sell_price: Optional[float] = None
    currency: str = "VND"
    data_source: str = "VN_MARKET"

class GoldHistoryItem(BaseModel):
    date: str
    nav: float
    open: float
    high: float
    low: float
    close: float
    adjclose: float
    volume: float = 0.0
    buy_price: Optional[float] = None
    sell_price: Optional[float] = None

class GoldHistoryResponse(BaseModel):
    symbol: str
    history: List[GoldHistoryItem]
    currency: str = "VND"
    data_source: str = "VN_MARKET"

