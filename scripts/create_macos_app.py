#!/usr/bin/env python3
"""
Create a standalone macOS .app bundle for distribution.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path


def create_standalone_app():
    """Create a standalone .app that includes Python and all dependencies."""
    
    print("🍎 Creating macOS App Bundle...")
    
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
    
    # Create a launcher script for the menu bar app
    launcher_script = """#!/usr/bin/env python3
import sys
import os

# Add the app bundle to Python path
bundle_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, bundle_dir)

# Import and run the menu bar app
from portify.menubar.app import run_menubar_app

if __name__ == "__main__":
    run_menubar_app()
"""
    
    with open('portify_launcher.py', 'w') as f:
        f.write(launcher_script)
    
    # PyInstaller command for macOS app bundle (simplified)
    cmd = [
        sys.executable, '-m', 'PyInstaller',
        '--name', 'Portify',
        '--windowed',  # No console window
        '--onedir',    # Directory bundle (not single file)
        '--clean',
        '--noconfirm',
        
        # Include data files
        '--add-data', 'portify:portify',
        
        # Icon
        '--icon', 'portify/menubar/icons/portify.icns',
        
        # Hidden imports for dependencies
        '--hidden-import', 'pystray._base',
        '--hidden-import', 'psutil',
        '--hidden-import', 'typer',
        '--hidden-import', 'rich',
        '--hidden-import', 'click',
        
        # macOS specific
        '--osx-bundle-identifier', 'com.portify.menubar',
        # Remove universal2 to avoid compatibility issues
        
        'portify_launcher.py'
    ]
    
    print(f"🔨 Building app bundle...")
    print("This may take a few minutes...")
    
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print("✅ App bundle created successfully!")
        
        # Check if the app was created
        app_path = Path("dist/Portify.app")
        if app_path.exists():
            print(f"📱 App location: {app_path.absolute()}")
            
            # Fix Info.plist to make it a menu bar agent
            fix_info_plist(app_path)
            
            # Get app size
            size_mb = get_directory_size(app_path) / (1024 * 1024)
            print(f"📊 App size: {size_mb:.1f} MB")
            
            # Create a DMG for distribution
            create_dmg()
            
            print("\n🎉 Ready for distribution!")
            print(f"📁 Upload to your website: {app_path}")
            
        else:
            print("❌ App bundle not found in expected location")
            
    except subprocess.CalledProcessError as e:
        print(f"❌ Build failed: {e}")
        if e.stdout:
            print(f"stdout: {e.stdout}")
        if e.stderr:
            print(f"stderr: {e.stderr}")
        return False
    
    finally:
        # Clean up launcher script
        if os.path.exists('portify_launcher.py'):
            os.remove('portify_launcher.py')
    
    return True


def fix_info_plist(app_path):
    """Fix the Info.plist to make the app a menu bar agent (no dock icon)."""
    import plistlib
    
    plist_path = app_path / "Contents" / "Info.plist"
    
    try:
        # Read existing plist
        with open(plist_path, 'rb') as f:
            plist_data = plistlib.load(f)
        
        # Add LSUIElement to hide from dock
        plist_data['LSUIElement'] = True
        plist_data['LSBackgroundOnly'] = False
        
        # Write back
        with open(plist_path, 'wb') as f:
            plistlib.dump(plist_data, f)
        
        print("✅ Fixed Info.plist - App will not show in dock")
        
    except Exception as e:
        print(f"⚠️  Could not fix Info.plist: {e}")


def get_directory_size(path):
    """Get total size of directory in bytes."""
    total = 0
    for dirpath, dirnames, filenames in os.walk(path):
        for filename in filenames:
            filepath = os.path.join(dirpath, filename)
            if os.path.exists(filepath):
                total += os.path.getsize(filepath)
    return total


def create_dmg():
    """Create a professional DMG file for easy distribution."""
    print("💿 Creating professional DMG file...")
    
    # Create a temporary directory for DMG contents
    dmg_temp_dir = "dist/dmg_temp"
    os.makedirs(dmg_temp_dir, exist_ok=True)
    
    try:
        # Copy the app to temp directory
        subprocess.run(['cp', '-R', 'dist/Portify.app', dmg_temp_dir], check=True)
        
        # Create Applications symlink for easy installation
        subprocess.run(['ln', '-s', '/Applications', f'{dmg_temp_dir}/Applications'], check=True)
        
        # Create README file with instructions
        readme_content = """🚀 Portify Installation

1. Drag Portify.app to the Applications folder
2. Open Portify from Applications or Spotlight
3. Look for the "P" icon in your menu bar
4. Click the icon to manage your ports!

Note: Portify runs in the menu bar only - no dock icon.

For help: https://github.com/your-repo/portify
"""
        
        with open(f'{dmg_temp_dir}/README.txt', 'w') as f:
            f.write(readme_content)
        
        # Create DMG
        dmg_cmd = [
            'hdiutil', 'create',
            '-volname', 'Portify Installer',
            '-srcfolder', dmg_temp_dir,
            '-ov', '-format', 'UDZO',
            'dist/Portify.dmg'
        ]
        
        subprocess.run(dmg_cmd, check=True, capture_output=True)
        print("✅ Professional DMG created: dist/Portify.dmg")
        
        # Get DMG size
        dmg_size = os.path.getsize('dist/Portify.dmg') / (1024 * 1024)
        print(f"📊 DMG size: {dmg_size:.1f} MB")
        
        # Clean up temp directory
        subprocess.run(['rm', '-rf', dmg_temp_dir])
        
    except subprocess.CalledProcessError as e:
        print(f"⚠️  Could not create DMG: {e}")
        # Clean up on error
        if os.path.exists(dmg_temp_dir):
            subprocess.run(['rm', '-rf', dmg_temp_dir])


if __name__ == "__main__":
    print("🚀 Portify Standalone App Builder")
    print("=" * 50)
    
    if sys.platform != 'darwin':
        print("❌ This script is for macOS only")
        print("💡 For Windows, use create_windows_exe.py")
        sys.exit(1)
    
    success = create_standalone_app()
    
    if success:
        print("\n📦 Distribution Files Created:")
        print("  • Portify.app - Drag & drop installer")
        print("  • Portify.dmg - Professional installer")
        print("\n🌐 Upload to your website:")
        print("  • Users download and double-click to install")
        print("  • No Python or dependencies required!")
    else:
        print("\n❌ Build failed")
        sys.exit(1)
