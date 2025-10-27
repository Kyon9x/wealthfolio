from vnstock import Fund
from datetime import datetime, timedelta
from typing import List, Dict, Optional
import logging
import pandas as pd

logger = logging.getLogger(__name__)

class FundClient:
    def __init__(self):
        self._funds_cache: Optional[List[Dict]] = None
        self._funds_map: Dict[str, int] = {}
        self._cache_timestamp: Optional[datetime] = None
        self._cache_duration = timedelta(hours=24)
        self._fund_api = Fund()
    
    def _is_cache_valid(self) -> bool:
        if self._cache_timestamp is None:
            return False
        return datetime.now() - self._cache_timestamp < self._cache_duration
    
    def _refresh_funds_cache(self):
        logger.info("Fetching fresh fund list from vnstock")
        funds_df = self._fund_api.listing()
        
        funds: List[Dict] = []
        self._funds_map = {}
        for _, row in funds_df.iterrows():
            fund_code = row.get("fund_code", "")
            short_name = row.get("short_name", "")
            fund_id_value = row.get("fund_id_fmarket", 0)
            fund_id = int(fund_id_value) if fund_id_value else 0
            funds.append({
                "symbol": short_name if short_name else fund_code,
                "fund_name": row.get("name", ""),
                "asset_type": "MUTUAL_FUND"
            })
            if fund_code:
                self._funds_map[fund_code.upper()] = fund_id
            if short_name:
                self._funds_map[short_name.upper()] = fund_id
        
        self._funds_cache = funds
        self._cache_timestamp = datetime.now()
        logger.info(f"Cached {len(funds)} funds")
    
    def get_funds_list(self) -> List[Dict]:
        if self._is_cache_valid() and self._funds_cache:
            logger.info("Using cached fund list")
            return self._funds_cache
        
        try:
            self._refresh_funds_cache()
            return self._funds_cache if self._funds_cache else []
        except Exception as e:
            logger.error(f"Error fetching funds list: {e}")
            if self._funds_cache:
                logger.warning("Returning stale cache due to error")
                return self._funds_cache
            raise
    
    def _get_fund_id(self, symbol: str) -> Optional[int]:
        if not self._is_cache_valid() or not self._funds_map:
            try:
                self._refresh_funds_cache()
            except Exception as e:
                logger.error(f"Error refreshing cache: {e}")
                return None
        
        return self._funds_map.get(symbol.upper())
    
    def search_fund_by_symbol(self, symbol: str) -> Optional[Dict]:
        try:
            fund_id = self._get_fund_id(symbol)
            if not fund_id:
                logger.warning(f"Fund ID not found for symbol: {symbol}")
                return None
            
            fund_info = self._fund_api.nav_report(fund_id)
            if fund_info is None or fund_info.empty:
                return None
            
            info = fund_info.iloc[-1]
            nav_value = info.get("nav_per_unit", 0.0)
            
            fund_name = symbol
            if self._funds_cache:
                for fund in self._funds_cache:
                    if fund.get("symbol", "").upper() == symbol.upper():
                        fund_name = fund.get("fund_name", symbol)
                        break
            
            result = {
                "symbol": symbol,
                "fund_name": fund_name,
                "fund_type": "MUTUAL_FUND",
                "management_company": "",
                "inception_date": "",
                "nav_per_unit": float(nav_value) if nav_value else 0.0,
            }
            return result
        except Exception as e:
            logger.error(f"Error searching fund {symbol}: {e}")
            return None
    
    def get_fund_nav_history(self, symbol: str, start_date: str, end_date: str) -> List[Dict]:
        try:
            fund_id = self._get_fund_id(symbol)
            if not fund_id:
                logger.warning(f"Fund ID not found for symbol: {symbol}")
                return []
            
            history_df = self._fund_api.nav_report(fund_id)
            
            if history_df is None or history_df.empty:
                return []
            
            history_df['date'] = pd.to_datetime(history_df['date'])
            history_df = history_df[(history_df['date'] >= start_date) & (history_df['date'] <= end_date)]
            
            history = []
            for _, row in history_df.iterrows():
                nav_value = row.get("nav_per_unit", 0.0)
                nav = float(nav_value) if nav_value else 0.0
                date_val = row.get("date")
                date_str = date_val.strftime("%Y-%m-%d") if isinstance(date_val, pd.Timestamp) else str(date_val)
                history.append({
                    "date": date_str,
                    "nav": nav,
                    "open": nav,
                    "high": nav,
                    "low": nav,
                    "close": nav,
                    "adjclose": nav,
                    "volume": 0.0
                })
            
            return history
        except Exception as e:
            logger.error(f"Error fetching NAV history for {symbol}: {e}")
            return []
    
    def get_latest_nav(self, symbol: str) -> Optional[Dict]:
        try:
            fund_id = self._get_fund_id(symbol)
            if not fund_id:
                logger.warning(f"Fund ID not found for symbol: {symbol}")
                return None
            
            nav_df = self._fund_api.nav_report(fund_id)
            if nav_df is None or nav_df.empty:
                return None
            
            info = nav_df.iloc[-1]
            date_val = info.get("date")
            if isinstance(date_val, pd.Timestamp):
                date_str = date_val.strftime("%Y-%m-%d")
            else:
                date_str = str(date_val) if date_val else datetime.now().strftime("%Y-%m-%d")
            
            nav_value = info.get("nav_per_unit", 0.0)
            return {
                "symbol": symbol,
                "nav": float(nav_value) if nav_value else 0.0,
                "date": date_str
            }
        except Exception as e:
            logger.error(f"Error fetching latest NAV for {symbol}: {e}")
            return None
