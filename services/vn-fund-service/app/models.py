from pydantic import BaseModel
from typing import List, Optional

class FundBasicInfo(BaseModel):
    symbol: str
    fund_name: str
    asset_type: str
    data_source: str = "VN_FUND"

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
    data_source: str = "VN_FUND"

class FundQuoteResponse(BaseModel):
    symbol: str
    nav: float
    date: str
    currency: str = "VND"
    data_source: str = "VN_FUND"

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
    data_source: str = "VN_FUND"

class HealthResponse(BaseModel):
    status: str
    service: str
    version: str
