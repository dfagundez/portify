#!/usr/bin/env python3
"""
Build script to create a standalone macOS app bundle for Portify.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path


def create_app_bundle():
    """Create a macOS app bundle for Portify."""
    
    print("🏗️  Building Portify.app...")
    
    # Check if PyInstaller is available
    try:
        import PyInstaller
    except ImportError:
        print("📦 Installing PyInstaller...")
        subprocess.run([sys.executable, '-m', 'pip', 'install', 'pyinstaller'], check=True)
    
    # Create the app bundle
    app_name = "Portify"
    
    # PyInstaller command
    cmd = [
        sys.executable, '-m', 'PyInstaller',
        '--name', app_name,
        '--windowed',  # No console window
        '--onedir',    # Create a directory bundle
        '--clean',
        '--noconfirm',
        '--add-data', 'portify/menubar/icons:portify/menubar/icons',
        '--hidden-import', 'pystray._base',
        '--hidden-import', 'PIL._tkinter_finder',
        '--osx-bundle-identifier', 'com.portify.menubar',
        'portify/main.py'
    ]
    
    print(f"🔨 Running: {' '.join(cmd)}")
    
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print("✅ App bundle created successfully!")
        
        # Create distribution directory
        dist_dir = Path("dist")
        app_path = dist_dir / f"{app_name}.app"
        
        if app_path.exists():
            print(f"📱 App bundle location: {app_path.absolute()}")
            print(f"💡 You can now drag {app_name}.app to your Applications folder")
            
            # Create a simple installer
            create_installer_script(app_name)
            
        else:
            print("❌ App bundle not found in expected location")
            
    except subprocess.CalledProcessError as e:
        print(f"❌ Build failed: {e}")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        return False
    
    return True


def create_installer_script(app_name):
    """Create a simple installer script."""
    
    installer_script = f"""#!/bin/bash

# Portify Installer Script

echo "🚀 Installing {app_name}..."

# Check if Applications directory is writable
if [ -w "/Applications" ]; then
    echo "📱 Copying {app_name}.app to Applications..."
    cp -R "dist/{app_name}.app" "/Applications/"
    echo "✅ {app_name} installed successfully!"
    echo "💡 You can now find {app_name} in your Applications folder"
    echo "🎯 Or run from Spotlight: Cmd+Space, type '{app_name}'"
else
    echo "⚠️  Cannot write to /Applications directory"
    echo "💡 Please drag {app_name}.app to your Applications folder manually"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "🚀 To start the menu bar app:"
echo "  1. Open {app_name} from Applications"
echo "  2. Or run: portify menubar"
echo ""
echo "📚 For help: portify --help"
"""
    
    with open("install_app.sh", "w") as f:
        f.write(installer_script)
    
    os.chmod("install_app.sh", 0o755)
    print("📜 Created install_app.sh script")


def create_dmg():
    """Create a DMG file for distribution."""
    print("💿 Creating DMG file...")
    
    # This would require additional tools like create-dmg
    # For now, we'll just mention it
    print("💡 To create a DMG file, you can use tools like:")
    print("   brew install create-dmg")
    print("   create-dmg --volname 'Portify' --window-pos 200 120 --window-size 600 300 Portify.dmg dist/")


if __name__ == "__main__":
    print("🚀 Portify App Builder")
    print("=" * 50)
    
    if sys.platform != 'darwin':
        print("❌ This script is designed for macOS only")
        sys.exit(1)
    
    success = create_app_bundle()
    
    if success:
        print("\n🎉 Build completed successfully!")
        print("\n📦 Distribution options:")
        print("  1. Use the generated .app bundle")
        print("  2. Run ./install_app.sh to install to Applications")
        print("  3. Create a DMG for distribution")
        
        create_dmg()
    else:
        print("\n❌ Build failed")
        sys.exit(1)
