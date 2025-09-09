# Changelog

All notable changes to Portify will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-09-09

### Added
- 🚀 **CLI Interface**: Complete command-line interface with Typer and Rich
  - `portify list` - List all active ports and processes
  - `portify kill <PID>` - Kill processes by PID
  - `portify monitor` - Real-time monitoring mode
  - `portify info` - System information
  - `portify version` - Version information

- 🎯 **Menu Bar App**: Native macOS menu bar application
  - Always-visible "P" icon in menu bar
  - One-click process killing
  - Auto-refresh every 5 seconds
  - Color-coded status indicators
  - Native macOS notifications
  - No dock icon (LSUIElement)

- 🎨 **Professional UI**: Beautiful, intuitive interface
  - Rich CLI output with tables and colors
  - Professional app icon with gradient design
  - Smart filtering and sorting
  - Status emojis and visual indicators

- 🔧 **Advanced Features**:
  - Cross-platform support (macOS and Linux)
  - Smart permission handling
  - Process information with CPU/memory usage
  - Port filtering by name, number, and status
  - Automatic update checking system

- 📦 **Distribution**:
  - Professional DMG installer for macOS
  - Standalone app bundle (no Python required)
  - Multiple installation methods
  - Comprehensive documentation

### Technical Details
- Built with Python 3.8+
- Uses psutil for cross-platform process management
- Typer for CLI framework
- Rich for beautiful terminal output
- pystray for menu bar functionality
- PyInstaller for app bundling

### Documentation
- Complete README with usage examples
- Menu bar app guide
- Client installation guide
- Development workflow documentation
- Distribution guide
