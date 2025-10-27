UPDATE market_data_providers
SET 
    id = 'VN_MARKET',
    name = 'VN Market Service',
    description = 'Market data provider for Vietnamese assets including stocks, mutual funds, indices, gold, forex, and cryptocurrencies. Provides historical and real-time data using vnstock library.',
    url = 'http://127.0.0.1:8765/docs'
WHERE id = 'VN_FUND';

UPDATE quotes
SET data_source = 'VN_MARKET'
WHERE data_source = 'VN_FUND';

UPDATE assets
SET data_source = 'VN_MARKET'
WHERE data_source = 'VN_FUND';
