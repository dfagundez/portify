# 🚀 Portify

A lightweight CLI tool for developers to quickly manage ports and processes on macOS and Linux.

## ✨ Features

- 📋 **List active ports** - See all network connections with process information
- ⚡ **Kill processes** - Terminate processes with a single command
- 🎨 **Beautiful interface** - Colorful, rich CLI output with tables
- 🔍 **Smart filtering** - Filter by process name, port number, or connection status
- 🔄 **Real-time monitoring** - Interactive mode with auto-refresh
- 🎯 **Menu Bar App** - Always-accessible icon in macOS menu bar
- 🛡️ **Permission handling** - Smart fallbacks for restricted access
- 💻 **Cross-platform** - Works on macOS and Linux
- 🚀 **Developer-focused** - Built for productivity and speed

## 🚀 Quick Start

### Easy Installation

```bash
# Clone the repository
git clone https://github.com/diegofagundez/portify.git
cd portify

# Run the installation script
./scripts/install.sh
```

### Manual Installation

```bash
# Install dependencies
python3 -m pip install -r requirements.txt

# Install Portify
python3 -m pip install -e .
```

### Usage Examples

```bash
# List all active ports and processes
portify list

# Show only listening ports
portify list --listening

# Filter by process name
portify list --filter chrome

# Filter by specific port
portify list --port 3000

# Include CPU and memory usage
portify list --system

# Kill a process by PID
portify kill 1234

# Kill with confirmation skip
portify kill 1234 --yes

# Force kill (SIGKILL)
portify kill 1234 --force

# Interactive monitoring mode
portify monitor

# Launch menu bar app (macOS)
portify menubar

# Show system information
portify info

# Show version
portify version
```

## 🛠️ Complete Command Reference

### `portify list`

List all active network connections and their processes.

**Options:**

- `--system, -s` - Include CPU and memory usage information
- `--filter, -f <name>` - Filter by process name
- `--port, -p <number>` - Filter by specific port number
- `--listening, -l` - Show only listening ports

**Examples:**

```bash
portify list                    # All connections
portify list --listening        # Only listening ports
portify list --filter node      # Only Node.js processes
portify list --port 3000        # Only port 3000
portify list --system           # Include system info
```

### `portify kill`

Kill a process by its PID.

**Options:**

- `--force, -f` - Use SIGKILL instead of SIGTERM
- `--yes, -y` - Skip confirmation prompt

**Examples:**

```bash
portify kill 1234              # Kill with confirmation
portify kill 1234 --yes        # Kill without confirmation
portify kill 1234 --force      # Force kill (SIGKILL)
```

### `portify monitor`

Interactive monitoring mode with real-time updates.

**Options:**

- `--interval, -i <seconds>` - Refresh interval (default: 2)
- `--system, -s` - Include CPU and memory usage

**Examples:**

```bash
portify monitor                 # Monitor with 2s interval
portify monitor --interval 5    # Monitor with 5s interval
portify monitor --system        # Monitor with system info
```

### `portify menubar`

Launch Portify as a menu bar application (macOS).

**Options:**

- `--max-ports, -m <number>` - Maximum ports to show in menu (default: 7)
- `--interval, -i <seconds>` - Refresh interval (default: 5)
- `--no-notifications` - Disable system notifications
- `--no-auto-refresh` - Disable automatic refresh

**Examples:**

```bash
portify menubar                 # Launch with default settings
portify menubar --max-ports 10  # Show up to 10 ports
portify menubar --interval 3    # Refresh every 3 seconds
```

**Features:**

- 🎯 Always-visible icon in menu bar
- 🔄 Auto-refresh every 5 seconds
- ❌ One-click process killing
- 🎨 Color-coded status indicators
- 🔔 System notifications
- 📋 Quick access to CLI

### `portify info`

Show system information and current user privileges.

### `portify version`

Show Portify version and information.

## 🎯 Menu Bar App (macOS)

The menu bar application provides always-accessible port management directly from your macOS menu bar.

### **Installation**

```bash
# Install menu bar dependencies
pip install -r requirements-menubar.txt

# Or install with extras
pip install -e ".[menubar]"

# Or use the installation script
./scripts/install-menubar.sh
```

### **Usage**

```bash
portify menubar  # Launch menu bar app
```

### **What You Get**

- **🎯 Menu Bar Icon**: Always-visible "P" icon with status colors
- **📊 Quick Overview**: See active ports at a glance
- **❌ One-Click Kill**: Terminate processes instantly
- **🔄 Auto-Refresh**: Updates every 5 seconds automatically
- **🔔 Notifications**: System alerts for important events
- **🎨 Smart Display**: Shows most important ports first

### **Icon Status Colors**

- 🔵 **Blue**: Normal operation
- 🟢 **Green**: Active listening ports
- 🟡 **Yellow**: High activity warning
- 🔴 **Red**: Issues detected
- ⚫ **Gray**: No active ports

For detailed menu bar app documentation, see [MENUBAR_GUIDE.md](docs/MENUBAR_GUIDE.md).

## 🔧 Development

### Setup Development Environment

```bash
# Clone the repository
git clone <repository-url>
cd portify

# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install in development mode
pip install -e .
```

### Project Structure

```
portify/
├── portify/
│   ├── __init__.py
│   ├── main.py              # Entry point
│   ├── core/
│   │   ├── port_scanner.py  # Port scanning logic
│   │   ├── process_manager.py # Process management
│   │   └── utils.py         # Utility functions
│   └── cli/
│       ├── commands.py      # CLI commands
│       └── display.py       # Rich display formatting
├── requirements.txt
├── setup.py
├── install.sh
└── README.md
```

### Dependencies

- **typer** - Modern CLI framework
- **rich** - Beautiful terminal output
- **psutil** - Cross-platform process utilities
- **click** - Command line interface creation

## 🚨 Permissions & Security

### macOS Considerations

On macOS, some operations may require additional permissions:

- **Full Disk Access** - For complete process information
- **Developer Tools** - May be required for some system calls
- **sudo privileges** - For killing system processes

### Linux Considerations

On Linux, you may need:

- **sudo privileges** - For killing processes owned by other users
- **proc filesystem access** - Usually available by default

### Security Notes

- Portify only reads process information and network connections
- Process termination requires appropriate permissions
- No data is transmitted or stored externally
- All operations are performed locally

## 🐛 Troubleshooting

### Common Issues

**"Access Denied" errors:**

- Try running with `sudo` for system processes
- On macOS, grant Full Disk Access in System Preferences

**"Process not found" errors:**

- Process may have already terminated
- PID may be invalid or expired

**Empty port list:**

- Check if you have permission to read network connections
- Try running with elevated privileges

**Installation issues:**

- Ensure Python 3.8+ is installed
- Make sure pip is up to date: `pip install --upgrade pip`

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Follow PEP 8 style guidelines
2. Add type hints where appropriate
3. Include docstrings for functions and classes
4. Test on both macOS and Linux when possible

## 📝 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Typer](https://typer.tiangolo.com/) for the CLI framework
- Styled with [Rich](https://rich.readthedocs.io/) for beautiful output
- Process management via [psutil](https://psutil.readthedocs.io/)

---

**Made with ❤️ for developers who need quick port management**
