import os

PORT = int(os.getenv("VN_MARKET_SERVICE_PORT", "8765"))
HOST = os.getenv("VN_MARKET_SERVICE_HOST", "127.0.0.1")
CORS_ORIGINS = ["tauri://localhost", "http://localhost:1420"]
