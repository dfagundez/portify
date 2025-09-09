"""
macOS-specific configuration for menu bar app.
"""

import os
import sys


def configure_macos_app():
    """Configure the app to run as a menu bar agent (no dock icon)."""
    try:
        # Only on macOS
        if sys.platform != 'darwin':
            return
        
        # Try to configure as LSUIElement (no dock icon)
        try:
            import AppKit
            # Set the app to be a UI Element (no dock icon, no menu bar)
            AppKit.NSApp.setActivationPolicy_(AppKit.NSApplicationActivationPolicyAccessory)
            print("✅ Configured as menu bar agent (no dock icon)")
        except ImportError:
            # Fallback: try with plistlib to create Info.plist
            print("⚠️  AppKit not available, app may show in dock")
            
    except Exception as e:
        print(f"⚠️  Could not configure macOS settings: {e}")


def hide_from_dock():
    """Alternative method to hide from dock using LSUIElement."""
    try:
        if sys.platform == 'darwin':
            import Foundation
            bundle = Foundation.NSBundle.mainBundle()
            if bundle:
                info = bundle.localizedInfoDictionary() or bundle.infoDictionary()
                if info:
                    info['LSUIElement'] = True
    except Exception:
        pass
