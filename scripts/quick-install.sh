#!/bin/bash

# Portify Quick Installer
# One-command installation for end users

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Banner
echo ""
echo "🚀 Portify Installer"
echo "===================="
echo "Port and Process Manager for Developers"
echo ""

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This installer is designed for macOS only."
    print_status "For Linux installation, please visit: https://github.com/your-repo/portify"
    exit 1
fi

print_status "Detected macOS system ✅"

# Check for Python 3
print_status "Checking Python installation..."

if ! command -v python3 &> /dev/null; then
    print_warning "Python 3 not found. Installing Python..."
    
    # Check if Homebrew is available
    if command -v brew &> /dev/null; then
        print_status "Installing Python via Homebrew..."
        brew install python3
    else
        print_error "Python 3 is required but not installed."
        echo ""
        echo "Please install Python 3 from one of these options:"
        echo "1. Download from: https://python.org/downloads/"
        echo "2. Install Homebrew first: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        echo "   Then run: brew install python3"
        echo ""
        echo "After installing Python, run this installer again."
        exit 1
    fi
fi

# Verify Python version
python_version=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
required_version="3.8"

if [ "$(printf '%s\n' "$required_version" "$python_version" | sort -V | head -n1)" != "$required_version" ]; then
    print_error "Python 3.8+ is required. You have Python $python_version"
    print_status "Please upgrade Python and try again."
    exit 1
fi

print_success "Python $python_version found ✅"

# Check if git is available
if ! command -v git &> /dev/null; then
    print_warning "Git not found. Installing Xcode Command Line Tools..."
    xcode-select --install
    print_status "Please complete the Xcode Command Line Tools installation and run this script again."
    exit 1
fi

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

print_status "Downloading Portify..."

# Clone repository (in real deployment, this would be from your actual repo)
if git clone https://github.com/your-username/portify.git; then
    print_success "Downloaded successfully ✅"
else
    print_error "Failed to download Portify"
    print_status "Please check your internet connection and try again."
    exit 1
fi

cd portify

print_status "Installing Portify..."

# Install base dependencies
print_status "Installing base dependencies..."
if python3 -m pip install -r requirements.txt --user; then
    print_success "Base dependencies installed ✅"
else
    print_error "Failed to install base dependencies"
    exit 1
fi

# Install menu bar dependencies
print_status "Installing menu bar dependencies..."
if python3 -m pip install -r requirements-menubar.txt --user; then
    print_success "Menu bar dependencies installed ✅"
else
    print_warning "Some menu bar dependencies failed to install"
    print_status "Portify will still work, but some features may be limited"
fi

# Install Portify
print_status "Installing Portify..."
if python3 -m pip install -e . --user; then
    print_success "Portify installed successfully ✅"
else
    print_error "Failed to install Portify"
    exit 1
fi

# Verify installation
print_status "Verifying installation..."
if command -v portify &> /dev/null; then
    print_success "Installation verified ✅"
else
    print_warning "Portify command not found in PATH"
    print_status "You may need to add ~/.local/bin to your PATH"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
    print_status "Added to ~/.zshrc - restart your terminal or run: source ~/.zshrc"
fi

# Clean up
cd /
rm -rf "$TEMP_DIR"

# Success message
echo ""
echo "🎉 Installation Complete!"
echo "========================"
echo ""
print_success "Portify is now installed on your system!"
echo ""
echo "🚀 Quick Start:"
echo "  portify list             # List all active ports"
echo "  portify menubar          # Launch menu bar app"
echo ""
echo "🎯 Menu Bar App:"
echo "  1. Run: portify menubar"
echo "  2. Look for the 'P' icon in your menu bar"
echo "  3. Click to see active ports and manage processes"
echo ""
echo "📚 Need help?"
echo "  portify --help           # Show all commands"
echo "  portify list --help      # Help for specific commands"
echo ""
echo "🐛 Issues? Report at: https://github.com/your-username/portify/issues"
echo ""

# Offer to start menu bar app
echo -n "🎯 Would you like to start the menu bar app now? [y/N]: "
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    print_status "Starting Portify menu bar app..."
    echo ""
    echo "💡 Look for the 'P' icon in your menu bar!"
    echo "⌨️  Press Ctrl+C to stop the app"
    echo ""
    
    # Start the menu bar app
    portify menubar
else
    print_status "You can start the menu bar app anytime with: portify menubar"
fi

echo ""
print_success "Thank you for using Portify! 🚀"
