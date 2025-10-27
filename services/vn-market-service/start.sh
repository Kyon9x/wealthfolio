#!/bin/bash

# VN Market Service Startup Script
# This script starts the VN Market Service on port 8765

cd "$(dirname "$0")"

# Check if service is already running
if lsof -i :8765 > /dev/null 2>&1; then
    echo "VN Market Service is already running on port 8765"
    exit 0
fi

# Determine which Python to use (prefer .venv, fallback to venv, then system python3)
if [ -d ".venv" ]; then
    PYTHON_BIN=".venv/bin/python"
elif [ -d "venv" ]; then
    PYTHON_BIN="venv/bin/python"
else
    PYTHON_BIN="python3"
fi

# Start the service (using exec to replace the shell process)
echo "Starting VN Market Service on http://127.0.0.1:8765"
exec $PYTHON_BIN -m uvicorn app.main:app --host 127.0.0.1 --port 8765 --log-level info
