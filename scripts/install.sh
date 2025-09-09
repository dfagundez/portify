#!/bin/bash

# Portify Installation Script
# This script installs Portify on macOS and Linux

set -e

echo "🚀 Installing Portify..."

# Check if Python 3 is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed."
    echo "Please install Python 3 and try again."
    exit 1
fi

# Check if pip is available
if ! python3 -m pip --version &> /dev/null; then
    echo "❌ pip is required but not available."
    echo "Please install pip and try again."
    exit 1
fi

echo "✅ Python 3 and pip found"

# Install dependencies
echo "📦 Installing dependencies..."
python3 -m pip install -r requirements.txt

# Install Portify in development mode
echo "🔧 Installing Portify..."
python3 -m pip install -e .

echo "✅ Installation complete!"
echo ""
echo "🎉 Portify is now installed!"
echo ""
echo "Try these commands:"
echo "  portify --help          # Show help"
echo "  portify list             # List all ports"
echo "  portify list --listening # Show only listening ports"
echo "  portify info             # Show system info"
echo ""
echo "For more information, visit: https://github.com/your-username/portify"
