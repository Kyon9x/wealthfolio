UPDATE market_data_providers
SET 
    id = 'VN_FUND',
    name = 'VN Fund Service',
    description = 'Market data provider for Vietnamese mutual funds. Provides historical and real-time data for Vietnamese investment funds.',
    url = 'http://127.0.0.1:8765/docs'
WHERE id = 'VN_MARKET';

UPDATE quotes
SET data_source = 'VN_FUND'
WHERE data_source = 'VN_MARKET';

UPDATE assets
SET data_source = 'VN_FUND'
WHERE data_source = 'VN_MARKET';
