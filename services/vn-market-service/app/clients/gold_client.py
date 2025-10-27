from vnstock.explorer.misc import sjc_gold_price
from datetime import datetime, timedelta
from typing import List, Dict, Optional
import logging
import pandas as pd

logger = logging.getLogger(__name__)

class GoldClient:
    def __init__(self):
        self.symbol = "VN_GOLD"
        self.name = "SJC Gold Price"
        # Fallback prices in VND per tael (1 tael ≈ 37.5 grams)
        # These are approximate SJC gold prices as of late 2024
        # Updated periodically or when API is available
        self.fallback_buy_price = 84_000_000.0  # 84 million VND per tael
        self.fallback_sell_price = 86_500_000.0  # 86.5 million VND per tael
    
    def get_gold_history(self, start_date: str, end_date: str) -> List[Dict]:
        """
        Fetch historical gold prices from SJC using vnstock API.
        
        The vnstock sjc_gold_price(date='YYYY-MM-DD') API returns historical data
        for the specified date. We call it for each date in the range to build
        a complete historical dataset.
        
        Falls back to fallback prices for dates where API fails.
        """
        try:
            # Parse date range
            start_dt = datetime.strptime(start_date, "%Y-%m-%d")
            end_dt = datetime.strptime(end_date, "%Y-%m-%d")
            
            # Build historical data by calling API for each date
            history = []
            current_dt = start_dt
            
            while current_dt <= end_dt:
                date_str = current_dt.strftime("%Y-%m-%d")
                
                # Try to fetch data for this specific date
                try:
                    df = sjc_gold_price(date=date_str)
                    
                    if df is not None and not df.empty:
                        # Take the first branch (usually the main SJC price)
                        # All branches typically have the same price
                        info = df.iloc[0]
                        
                        buy_price = float(info.get("buy_price", 0.0)) if not pd.isna(info.get("buy_price")) else 0.0
                        sell_price = float(info.get("sell_price", 0.0)) if not pd.isna(info.get("sell_price")) else 0.0
                        
                        # Use sell_price as the main price (close price equivalent)
                        price = sell_price if sell_price > 0 else buy_price
                        
                        if price > 0:
                            # For gold, we don't have true OHLC data
                            # Use the same price for all fields (daily settlement price)
                            history.append({
                                "date": date_str,
                                "nav": price,
                                "open": price,
                                "high": price,
                                "low": price,
                                "close": price,
                                "adjclose": price,
                                "volume": 0.0,  # No volume data for gold
                                "buy_price": buy_price,
                                "sell_price": sell_price
                            })
                            current_dt += timedelta(days=1)
                            continue
                    
                    # If we get here, data was empty or invalid - use fallback
                    logger.warning(f"No data from API for {date_str}, using fallback")
                    
                except Exception as date_error:
                    # API failed for this date - use fallback
                    logger.warning(f"API error for {date_str}: {date_error}, using fallback")
                
                # Fallback: use hardcoded prices for this date
                price = self.fallback_sell_price
                history.append({
                    "date": date_str,
                    "nav": price,
                    "open": price,
                    "high": price,
                    "low": price,
                    "close": price,
                    "adjclose": price,
                    "volume": 0.0,
                    "buy_price": self.fallback_buy_price,
                    "sell_price": self.fallback_sell_price
                })
                
                current_dt += timedelta(days=1)
            
            return history
            
        except Exception as e:
            logger.error(f"Error fetching gold history: {e}")
            return []
    
    def get_latest_quote(self) -> Optional[Dict]:
        """
        Fetch the latest gold price from SJC.
        Returns the current buy and sell prices.
        Falls back to hardcoded prices if API is unavailable.
        """
        try:
            df = sjc_gold_price()
            
            if df is None or df.empty:
                logger.warning("vnstock API returned empty data, using fallback prices")
                return self._get_fallback_quote()
            
            # Get the most recent entry (usually aggregate across branches)
            # We'll take the first row which typically represents the main SJC price
            info = df.iloc[0]
            
            date_val = info.get("date")
            if isinstance(date_val, pd.Timestamp):
                date_str = date_val.strftime("%Y-%m-%d")
            else:
                date_str = str(date_val) if date_val else datetime.now().strftime("%Y-%m-%d")
            
            buy_price = float(info.get("buy_price", 0.0)) if not pd.isna(info.get("buy_price")) else 0.0
            sell_price = float(info.get("sell_price", 0.0)) if not pd.isna(info.get("sell_price")) else 0.0
            
            # Use sell_price as the main price (close price equivalent)
            close_price = sell_price if sell_price > 0 else buy_price
            
            return {
                "symbol": self.symbol,
                "close": close_price,
                "date": date_str,
                "buy_price": buy_price,
                "sell_price": sell_price
            }
        except Exception as e:
            logger.warning(f"Error fetching latest gold quote from API: {e}, using fallback prices")
            return self._get_fallback_quote()
    
    def _get_fallback_quote(self) -> Dict:
        """
        Return fallback gold prices when API is unavailable.
        Uses hardcoded approximate SJC gold prices.
        """
        today = datetime.now().strftime("%Y-%m-%d")
        close_price = self.fallback_sell_price
        
        return {
            "symbol": self.symbol,
            "close": close_price,
            "date": today,
            "buy_price": self.fallback_buy_price,
            "sell_price": self.fallback_sell_price
        }
    
    def search_gold(self) -> Optional[Dict]:
        """
        Return gold asset information for search results.
        Always returns the gold asset info since we have fallback prices.
        """
        try:
            # Always return gold info since we have fallback prices
            return {
                "symbol": self.symbol,
                "name": self.name,
                "asset_type": "Commodity",
                "exchange": "SJC",
                "currency": "VND"
            }
        except Exception as e:
            logger.error(f"Error searching gold: {e}")
            return None
