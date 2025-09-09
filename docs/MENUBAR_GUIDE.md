# 🎯 Portify Menu Bar App Guide

## 🚀 Quick Start

### Installation
```bash
# Install menu bar dependencies
pip install -r requirements-menubar.txt

# Or install with extras
pip install -e ".[menubar]"
```

### Launch Menu Bar App
```bash
portify menubar
```

## 🎨 What You'll See

### 1. **Menu Bar Icon**
Look for the **"P"** icon in your macOS menu bar (top-right area)

### 2. **Icon Colors**
- 🔵 **Blue (Normal)**: Everything is running smoothly
- 🟢 **Green (Active)**: Ports are actively listening
- 🟡 **Yellow (Warning)**: Many ports active or high activity
- 🔴 **Red (Error)**: Issues detected
- ⚫ **Gray (Inactive)**: No active ports

### 3. **Menu Structure**
```
🚀 Portify (X connections)
─────────────────────────
📊 Y listening, X total
─────────────────────────
🟢 Postman (15611)        ▶ Kill Process (PID: 563)
                            Protocol: TCP
                            Status: LISTEN
                            Address: :::15611
🔵 Chrome (52133)         ▶ Kill Process (PID: 800)
🟡 Node.js (3000)         ▶ Kill Process (PID: 1234)
... and 5 more
─────────────────────────
🔄 Refresh Now
📋 Open CLI
⚙️ Auto-refresh: ON
─────────────────────────
🚪 Quit Portify
```

## ⚡ Features

### **Smart Port Display**
- Shows **listening ports first** (most important)
- **Limits to 7 ports** by default (configurable)
- **Color-coded status** indicators
- **Hover for details** (protocol, address, status)

### **One-Click Actions**
- **❌ Kill Process**: Click to terminate any process
- **🔄 Refresh**: Manual refresh of port list
- **📋 Open CLI**: Opens Terminal with `portify list`
- **⚙️ Toggle Auto-refresh**: Enable/disable automatic updates

### **Smart Notifications**
- Process killed successfully ✅
- Kill operation failed ❌
- Permission errors 🔒
- System status changes 📊

### **Auto-Refresh**
- Updates every **5 seconds** by default
- Can be disabled via menu or command line
- Efficient background scanning

## 🛠️ Command Line Options

```bash
# Basic usage
portify menubar

# Custom configuration
portify menubar --max-ports 10 --interval 3

# Disable features
portify menubar --no-notifications --no-auto-refresh

# Help
portify menubar --help
```

### **Options Explained**
- `--max-ports, -m`: Maximum ports shown in menu (default: 7)
- `--interval, -i`: Refresh interval in seconds (default: 5)
- `--no-notifications`: Disable system notifications
- `--no-auto-refresh`: Disable automatic refresh

## 🎯 Usage Scenarios

### **Development Workflow**
1. **Start your dev servers** (React, Node.js, etc.)
2. **Launch Portify**: `portify menubar`
3. **Quick monitoring**: Glance at menu bar icon
4. **Kill stuck processes**: Click ❌ next to any process
5. **Check details**: Hover over processes for info

### **System Monitoring**
- **Icon color changes** indicate system status
- **Notification alerts** for important events
- **Quick access** without opening Terminal

### **Troubleshooting**
- **Port conflicts**: See which process is using a port
- **Stuck processes**: Kill them with one click
- **System overview**: Quick count of active connections

## 🔧 Troubleshooting

### **Icon Not Appearing**
1. Check if app is running: `ps aux | grep portify`
2. Try restarting: Kill process and run `portify menubar` again
3. Check permissions in System Preferences

### **"Access Denied" Errors**
- Some processes require **sudo privileges**
- Grant **Full Disk Access** in System Preferences > Privacy & Security
- Run with elevated privileges if needed

### **Menu Not Updating**
- Check if auto-refresh is enabled (⚙️ in menu)
- Try manual refresh (🔄 in menu)
- Restart the app if issues persist

### **Notifications Not Working**
- Check System Preferences > Notifications
- Allow notifications for Terminal/Python
- Use `--no-notifications` flag if problematic

## 🚪 Stopping the App

### **From Menu**
Click **🚪 Quit Portify** in the menu

### **From Terminal**
Press **Ctrl+C** in the terminal where you started it

### **Force Kill**
```bash
pkill -f "portify menubar"
```

## 💡 Pro Tips

1. **Pin to Login Items**: Add to macOS login items for auto-start
2. **Keyboard Shortcuts**: Use Spotlight to quickly launch `portify menubar`
3. **Multiple Instances**: Don't run multiple instances simultaneously
4. **Resource Usage**: Very lightweight, minimal CPU/memory impact
5. **Integration**: Works perfectly alongside CLI commands

## 🔄 Auto-Start on Login

### **Option 1: Manual Setup**
1. Open **System Preferences** > **Users & Groups**
2. Click **Login Items**
3. Add a script that runs `portify menubar`

### **Option 2: LaunchAgent (Advanced)**
Create a LaunchAgent plist file for automatic startup.

---

**🎉 Enjoy your new menu bar port manager!**

For more information, run `portify --help` or check the main README.md
