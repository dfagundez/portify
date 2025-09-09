#!/usr/bin/env python3
"""
Create Info.plist for macOS app to configure it as menu bar agent.
"""

import plistlib
import os


def create_info_plist():
    """Create Info.plist that configures app as menu bar agent (no dock icon)."""
    
    plist_data = {
        'CFBundleDisplayName': 'Portify',
        'CFBundleName': 'Portify',
        'CFBundleIdentifier': 'com.portify.menubar',
        'CFBundleVersion': '1.0.0',
        'CFBundleShortVersionString': '1.0.0',
        'CFBundleInfoDictionaryVersion': '6.0',
        'CFBundlePackageType': 'APPL',
        'CFBundleSignature': 'PORT',
        'CFBundleExecutable': 'Portify',
        'CFBundleIconFile': 'portify.icns',
        
        # This is the key setting - makes it a menu bar agent (no dock icon)
        'LSUIElement': True,
        
        # Background processing
        'LSBackgroundOnly': False,
        
        # Supported architectures
        'LSArchitecturePriority': ['x86_64', 'arm64'],
        
        # Minimum macOS version
        'LSMinimumSystemVersion': '10.13.0',
        
        # Category
        'LSApplicationCategoryType': 'public.app-category.developer-tools',
        
        # Copyright
        'NSHumanReadableCopyright': 'Copyright © 2024 Portify. All rights reserved.',
        
        # High resolution capable
        'NSHighResolutionCapable': True,
        
        # Permissions (for accessing process information)
        'NSAppleEventsUsageDescription': 'Portify needs access to system events to manage processes.',
        
        # URL schemes (for future updates)
        'CFBundleURLTypes': [
            {
                'CFBundleURLName': 'com.portify.menubar',
                'CFBundleURLSchemes': ['portify']
            }
        ]
    }
    
    # Write Info.plist
    plist_path = 'Info.plist'
    with open(plist_path, 'wb') as f:
        plistlib.dump(plist_data, f)
    
    print(f"✅ Created {plist_path}")
    print("🚫 App will NOT appear in dock (LSUIElement = True)")
    print("🎯 App will only show in menu bar")
    
    return plist_path


if __name__ == "__main__":
    create_info_plist()
