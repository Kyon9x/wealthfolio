from vnstock import Quote, Listing
from datetime import datetime
from typing import List, Dict, Optional
import logging
import pandas as pd

logger = logging.getLogger(__name__)

class StockClient:
    def __init__(self):
        self._quote = None
        self._listing = Listing()
    
    def get_stock_history(self, symbol: str, start_date: str, end_date: str) -> List[Dict]:
        try:
            quote = Quote(symbol=symbol, source='VCI')
            history_df = quote.history(start=start_date, end=end_date)
            
            if history_df is None or history_df.empty:
                return []
            
            history = []
            for _, row in history_df.iterrows():
                date_val = row.get("time") or row.get("tradingDate")
                if pd.isna(date_val):
                    continue
                    
                date_str = date_val.strftime("%Y-%m-%d") if isinstance(date_val, pd.Timestamp) else str(date_val)
                
                # Convert from shortened VND format (e.g., 12) to actual VND (e.g., 12000)
                open_val = float(row.get("open", 0.0)) * 1000 if not pd.isna(row.get("open")) else 0.0
                high_val = float(row.get("high", 0.0)) * 1000 if not pd.isna(row.get("high")) else 0.0
                low_val = float(row.get("low", 0.0)) * 1000 if not pd.isna(row.get("low")) else 0.0
                close_val = float(row.get("close", 0.0)) * 1000 if not pd.isna(row.get("close")) else 0.0
                volume_val = float(row.get("volume", 0.0)) if not pd.isna(row.get("volume")) else 0.0
                
                history.append({
                    "date": date_str,
                    "nav": close_val,
                    "open": open_val,
                    "high": high_val,
                    "low": low_val,
                    "close": close_val,
                    "adjclose": close_val,
                    "volume": volume_val
                })
            
            return history
        except Exception as e:
            logger.error(f"Error fetching stock history for {symbol}: {e}")
            return []
    
    def get_latest_quote(self, symbol: str) -> Optional[Dict]:
        try:
            today = datetime.now().strftime("%Y-%m-%d")
            quote = Quote(symbol=symbol, source='VCI')
            quote_df = quote.history(start=today, end=today)
            
            if quote_df is None or quote_df.empty:
                return None
            
            info = quote_df.iloc[-1]
            date_val = info.get("time") or info.get("tradingDate")
            if isinstance(date_val, pd.Timestamp):
                date_str = date_val.strftime("%Y-%m-%d")
            else:
                date_str = str(date_val) if date_val else datetime.now().strftime("%Y-%m-%d")
            
            # Convert from shortened VND format (e.g., 12) to actual VND (e.g., 12000)
            close_val = float(info.get("close", 0.0)) * 1000 if not pd.isna(info.get("close")) else 0.0
            
            return {
                "symbol": symbol,
                "close": close_val,
                "date": date_str
            }
        except Exception as e:
            logger.error(f"Error fetching latest quote for {symbol}: {e}")
            return None
    
    def search_stock(self, symbol: str) -> Optional[Dict]:
        try:
            # Try to get company info from listing
            companies_df = self._listing.all_symbols()
            if companies_df is not None and not companies_df.empty:
                company_row = companies_df[companies_df['symbol'] == symbol]
                if not company_row.empty:
                    info = company_row.iloc[0]
                    company_name = str(info.get("organ_name", symbol))
                    
                    return {
                        "symbol": symbol,
                        "company_name": company_name,
                        "exchange": "HOSE",
                        "industry": "",
                        "company_type": ""
                    }
            
            # Stock not found in listing
            return None
        except Exception as e:
            logger.error(f"Error searching stock {symbol}: {e}")
            return None
