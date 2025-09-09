# 🎯 Portify

**The fastest way to manage ports and processes on macOS** - Always accessible from your menu bar, no terminal required.

![Portify Menu Bar Demo](https://via.placeholder.com/600x300/2563eb/ffffff?text=Portify+Menu+Bar+Demo)

## ✨ Why Portify?

**Stop switching to terminal every time you need to kill a process.** Portify lives in your menu bar, giving you instant access to:

- 🎯 **One-click process killing** - See what's running, click to kill
- 🔄 **Real-time updates** - Always shows current port status
- 🎨 **Smart visual indicators** - Color-coded status at a glance
- 🔔 **Native notifications** - Get alerts when processes change
- ⚡ **Zero setup** - Install once, always available
- 🛡️ **Permission smart** - Handles macOS security gracefully

## 🚀 Core Features

### 🎯 **Menu Bar App** (Primary Interface)

- **Always visible** - "P" icon in your macOS menu bar
- **One-click killing** - No commands to remember
- **Auto-refresh** - Updates every 5 seconds
- **Smart display** - Shows most important ports first
- **Native notifications** - System alerts for events
- **Color-coded status** - Visual health indicators

### 💻 **CLI Interface** (Power Users)

- **Rich terminal output** - Beautiful tables and colors
- **Advanced filtering** - By process, port, or status
- **Batch operations** - Kill multiple processes
- **System monitoring** - CPU and memory usage
- **Cross-platform** - Works on macOS and Linux

## 🚀 Quick Start

### 📥 **Download & Install** (Recommended)

1. **Download** the latest `.dmg` from [Releases](https://github.com/dfagundez/portify/releases)
2. **Double-click** `Portify.dmg`
3. **Drag** `Portify.app` to your Applications folder
4. **Launch** Portify from Applications
5. **Look for the "P" icon** in your menu bar!

### 🎯 **Using the Menu Bar App**

Once installed, Portify runs automatically in your menu bar:

1. **Click the "P" icon** in your menu bar
2. **See all active ports** and their processes
3. **Click "Kill"** next to any process to terminate it
4. **Get notifications** when processes start/stop

**That's it!** No terminal commands needed.

### 🔧 **Developer Installation** (CLI + Menu Bar)

For developers who want both interfaces:

```bash
# Clone and install
git clone https://github.com/dfagundez/portify.git
cd portify
./scripts/install.sh

# Launch menu bar app
portify menubar

# Or use CLI commands
portify list
portify kill 1234
```

## 🎯 **Menu Bar App Guide**

### **What You See**

- **🎯 Menu Bar Icon**: Always-visible "P" with status colors
- **📊 Active Ports List**: Current network connections
- **❌ Kill Buttons**: One-click process termination
- **🔄 Auto-refresh**: Updates every 5 seconds
- **🔔 Notifications**: Alerts for important events

### **Icon Status Colors**

- 🔵 **Blue**: Normal operation (default)
- 🟢 **Green**: Active listening ports detected
- 🟡 **Yellow**: High activity warning
- 🔴 **Red**: Issues or blocked ports detected
- ⚫ **Gray**: No active ports found

### **Menu Options**

- **Refresh Now** - Manual refresh
- **Open CLI** - Launch terminal interface
- **Settings** - Configure refresh rate and notifications
- **Quit** - Close Portify

## 💻 **CLI Interface** (For Power Users)

The CLI provides advanced features for developers who prefer terminal workflows:

### **Quick CLI Examples**

```bash
# Launch menu bar app (most common)
portify menubar

# List all active ports
portify list

# Kill a specific process
portify kill 1234

# Monitor in real-time
portify monitor

# Filter by process name
portify list --filter chrome

# Show system information
portify info
```

### **Advanced CLI Commands**

<details>
<summary><strong>🔍 portify list</strong> - List active connections</summary>

**Options:**

- `--system, -s` - Include CPU and memory usage
- `--filter, -f <name>` - Filter by process name
- `--port, -p <number>` - Filter by specific port
- `--listening, -l` - Show only listening ports

**Examples:**

```bash
portify list --listening        # Only listening ports
portify list --filter node      # Only Node.js processes
portify list --port 3000        # Only port 3000
portify list --system           # Include system info
```

</details>

<details>
<summary><strong>⚡ portify kill</strong> - Terminate processes</summary>

**Options:**

- `--force, -f` - Use SIGKILL instead of SIGTERM
- `--yes, -y` - Skip confirmation prompt

**Examples:**

```bash
portify kill 1234              # Kill with confirmation
portify kill 1234 --yes        # Kill without confirmation
portify kill 1234 --force      # Force kill (SIGKILL)
```

</details>

<details>
<summary><strong>🔄 portify monitor</strong> - Real-time monitoring</summary>

**Options:**

- `--interval, -i <seconds>` - Refresh interval (default: 2)
- `--system, -s` - Include CPU and memory usage

**Examples:**

```bash
portify monitor                 # Monitor with 2s interval
portify monitor --interval 5    # Monitor with 5s interval
portify monitor --system        # Monitor with system info
```

</details>

<details>
<summary><strong>🎯 portify menubar</strong> - Launch menu bar app</summary>

**Options:**

- `--max-ports, -m <number>` - Maximum ports to show (default: 7)
- `--interval, -i <seconds>` - Refresh interval (default: 5)
- `--no-notifications` - Disable system notifications
- `--no-auto-refresh` - Disable automatic refresh

**Examples:**

```bash
portify menubar                 # Launch with defaults
portify menubar --max-ports 10  # Show up to 10 ports
portify menubar --interval 3    # Refresh every 3 seconds
```

</details>

## 🛠️ **Installation Options**

### 📦 **Option 1: Download App** (Easiest)

Perfect for regular users who just want the menu bar app:

1. Go to [Releases](https://github.com/dfagundez/portify/releases)
2. Download `Portify.dmg`
3. Install like any macOS app
4. Launch and enjoy!

### 🔧 **Option 2: Developer Install** (Full Features)

For developers who want both menu bar + CLI:

```bash
# Clone and install everything
git clone https://github.com/dfagundez/portify.git
cd portify
./scripts/install.sh

# Launch menu bar app
portify menubar
```

### ⚡ **Option 3: CLI Only** (Minimal)

If you only want command-line interface:

```bash
pip install git+https://github.com/dfagundez/portify.git
portify list
```

## 🔧 **Development**

### **Contributing Setup**

```bash
# Clone and setup development environment
git clone https://github.com/dfagundez/portify.git
cd portify

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install in development mode
pip install -e ".[menubar]"
```

### **Project Architecture**

```
portify/
├── portify/
│   ├── menubar/           # 🎯 Menu Bar App (Primary)
│   │   ├── app.py         # Main menu bar application
│   │   ├── menu_manager.py # Menu logic and UI
│   │   ├── notifications.py # Native macOS notifications
│   │   └── icons/         # App icons and assets
│   ├── cli/               # 💻 CLI Interface (Secondary)
│   │   ├── commands.py    # Terminal commands
│   │   └── display.py     # Rich formatting
│   └── core/              # 🔧 Shared Logic
│       ├── port_scanner.py # Port scanning engine
│       └── process_manager.py # Process management
├── scripts/               # 📦 Build and distribution
└── docs/                  # 📚 Documentation
```

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

## 🎯 **Why Choose Portify?**

### **Before Portify:**

```bash
# Every time you need to kill a process:
$ lsof -i :3000
$ ps aux | grep node
$ kill -9 1234
# Repeat this dance 10+ times per day...
```

### **With Portify:**

1. **Click "P" in menu bar** 👆
2. **Click "Kill" next to process** ❌
3. **Done!** ✅

**Save 30+ seconds every time. That's hours per week.**

---

**Made with ❤️ for developers who value their time**
