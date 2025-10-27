#!/bin/bash

# VN Market Service Startup Script
# This script starts the VN Market Service on port 8765

cd "$(dirname "$0")"

# Check if service is already running
if lsof -i :8765 > /dev/null 2>&1; then
    echo "VN Market Service is already running on port 8765"
    exit 0
fi

# Activate virtual environment if it exists (prefer .venv, fallback to venv)
if [ -d ".venv" ]; then
    source .venv/bin/activate
elif [ -d "venv" ]; then
    source venv/bin/activate
fi

# Start the service
echo "Starting VN Market Service on http://127.0.0.1:8765"
python3 -m uvicorn app.main:app --host 127.0.0.1 --port 8765 --log-level info
