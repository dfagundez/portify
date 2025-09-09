#!/bin/bash

# Portify Menu Bar Installation Script
# This script installs Portify with menu bar functionality

set -e

echo "🚀 Installing Portify with Menu Bar support..."

# Check if Python 3 is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed."
    echo "Please install Python 3 from https://python.org and try again."
    exit 1
fi

# Check Python version (need 3.8+)
python_version=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
required_version="3.8"

if [ "$(printf '%s\n' "$required_version" "$python_version" | sort -V | head -n1)" != "$required_version" ]; then
    echo "❌ Python 3.8+ is required. You have Python $python_version"
    exit 1
fi

echo "✅ Python $python_version found"

# Install base dependencies
echo "📦 Installing base dependencies..."
python3 -m pip install -r requirements.txt

# Install menu bar dependencies
echo "🎯 Installing menu bar dependencies..."
python3 -m pip install -r requirements-menubar.txt

# Install Portify
echo "🔧 Installing Portify..."
python3 -m pip install -e .

echo ""
echo "✅ Installation complete!"
echo ""
echo "🎉 Portify is now installed with Menu Bar support!"
echo ""
echo "🚀 Quick Start:"
echo "  portify list             # CLI interface"
echo "  portify menubar          # Menu bar app"
echo ""
echo "💡 Menu Bar App:"
echo "  - Look for the 'P' icon in your menu bar"
echo "  - Click to see active ports and kill processes"
echo "  - Auto-refreshes every 5 seconds"
echo ""
echo "📚 Documentation:"
echo "  - README.md for CLI usage"
echo "  - MENUBAR_GUIDE.md for menu bar app"
echo ""
echo "🆘 Need help? Run: portify --help"
