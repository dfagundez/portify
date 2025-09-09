#!/usr/bin/env python3
"""
Create a standalone Windows .exe for distribution.
Note: This should be run on a Windows machine or Windows VM.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path


def create_windows_exe():
    """Create a standalone Windows executable."""
    
    print("🪟 Creating Windows Executable...")
    
    # Install PyInstaller if not available
    try:
        import PyInstaller
    except ImportError:
        print("📦 Installing PyInstaller...")
        subprocess.run([sys.executable, '-m', 'pip', 'install', 'pyinstaller'], check=True)
    
    # Clean previous builds
    if os.path.exists('dist'):
        shutil.rmtree('dist')
    if os.path.exists('build'):
        shutil.rmtree('build')
    
    # Create a Windows launcher script
    launcher_script = """#!/usr/bin/env python3
import sys
import os
import tkinter as tk
from tkinter import messagebox

# Add the executable directory to Python path
exe_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, exe_dir)

def show_error(title, message):
    root = tk.Tk()
    root.withdraw()
    messagebox.showerror(title, message)
    root.destroy()

def show_info(title, message):
    root = tk.Tk()
    root.withdraw()
    messagebox.showinfo(title, message)
    root.destroy()

try:
    # Import and run CLI (Windows doesn't have menu bar like macOS)
    from portify.cli.commands import app
    
    # Show info about CLI usage
    show_info("Portify Started", 
              "Portify CLI is now available!\\n\\n"
              "Open Command Prompt and use:\\n"
              "• portify list - List active ports\\n"
              "• portify kill <PID> - Kill process\\n"
              "• portify --help - Show all commands")
    
    # Run the CLI
    app()
    
except Exception as e:
    show_error("Portify Error", f"Failed to start Portify:\\n{str(e)}")
    sys.exit(1)
"""
    
    with open('portify_windows_launcher.py', 'w') as f:
        f.write(launcher_script)
    
    # PyInstaller command for Windows
    cmd = [
        sys.executable, '-m', 'PyInstaller',
        '--name', 'Portify',
        '--onefile',   # Single executable
        '--clean',
        '--noconfirm',
        
        # Icon (if you have one)
        # '--icon', 'portify/icons/portify.ico',
        
        # Include data files
        '--add-data', 'portify;portify',
        '--add-data', 'requirements.txt;.',
        
        # Hidden imports
        '--hidden-import', 'psutil',
        '--hidden-import', 'typer',
        '--hidden-import', 'rich',
        '--hidden-import', 'click',
        
        # Windows specific
        '--console',  # Keep console for CLI usage
        
        'portify_windows_launcher.py'
    ]
    
    print(f"🔨 Building Windows executable...")
    print("This may take a few minutes...")
    
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print("✅ Windows executable created successfully!")
        
        # Check if the exe was created
        exe_path = Path("dist/Portify.exe")
        if exe_path.exists():
            print(f"💻 Executable location: {exe_path.absolute()}")
            
            # Get exe size
            size_mb = exe_path.stat().st_size / (1024 * 1024)
            print(f"📊 Executable size: {size_mb:.1f} MB")
            
            print("\n🎉 Ready for distribution!")
            print(f"📁 Upload to your website: {exe_path}")
            
        else:
            print("❌ Executable not found in expected location")
            
    except subprocess.CalledProcessError as e:
        print(f"❌ Build failed: {e}")
        if e.stdout:
            print(f"stdout: {e.stdout}")
        if e.stderr:
            print(f"stderr: {e.stderr}")
        return False
    
    finally:
        # Clean up launcher script
        if os.path.exists('portify_windows_launcher.py'):
            os.remove('portify_windows_launcher.py')
    
    return True


def create_installer():
    """Create a Windows installer using NSIS (if available)."""
    print("📦 Creating Windows installer...")
    
    # This would require NSIS (Nullsoft Scriptable Install System)
    # For now, just provide the .exe file
    print("💡 For a professional installer, consider using:")
    print("  • NSIS (Nullsoft Scriptable Install System)")
    print("  • Inno Setup")
    print("  • WiX Toolset")


if __name__ == "__main__":
    print("🚀 Portify Windows Executable Builder")
    print("=" * 50)
    
    if sys.platform == 'darwin':
        print("⚠️  You're on macOS. This script creates Windows executables.")
        print("💡 You can still run it, but test the .exe on Windows.")
    
    success = create_windows_exe()
    
    if success:
        print("\n📦 Distribution Files Created:")
        print("  • Portify.exe - Windows executable")
        print("\n🌐 Upload to your website:")
        print("  • Users download and double-click to run")
        print("  • No Python installation required!")
        print("\n💡 Note: Windows users will use CLI commands")
        print("  (Windows doesn't have menu bar like macOS)")
    else:
        print("\n❌ Build failed")
        sys.exit(1)
