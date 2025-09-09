"""
Main menu bar application for Portify.
"""

import os
import sys
import threading
import time
from typing import Optional

import pystray
from PIL import Image

from .menu_manager import MenuManager, MenuConfig
from .icon_generator import create_portify_icon
from .macos_config import configure_macos_app


class PortifyMenuBarApp:
    """Main menu bar application class."""
    
    def __init__(self, config: Optional[MenuConfig] = None):
        self.config = config or MenuConfig()
        self.menu_manager = MenuManager(self.config)
        self.icon: Optional[pystray.Icon] = None
        self.is_running = False
        
        # Load icons
        self.icons = self._load_icons()
        
    def _load_icons(self) -> dict:
        """Load all status icons."""
        icons_dir = os.path.join(os.path.dirname(__file__), 'icons')
        icons = {}
        
        statuses = ['normal', 'active', 'warning', 'error', 'inactive']
        
        for status in statuses:
            icon_path = os.path.join(icons_dir, f'portify_{status}.png')
            try:
                if os.path.exists(icon_path):
                    icons[status] = Image.open(icon_path)
                else:
                    # Generate icon if file doesn't exist
                    icons[status] = create_portify_icon(status)
            except Exception as e:
                print(f"Warning: Could not load icon for {status}: {e}")
                # Fallback to generated icon
                icons[status] = create_portify_icon(status)
        
        return icons
    
    def _get_current_icon(self) -> Image.Image:
        """Get the appropriate icon based on current status."""
        status = self.menu_manager.get_current_status()
        return self.icons.get(status, self.icons['normal'])
    
    def _create_icon(self) -> pystray.Icon:
        """Create the pystray Icon object."""
        icon = pystray.Icon(
            name="Portify",
            icon=self._get_current_icon(),
            title="Portify - Port Manager",
            menu=self.menu_manager.create_menu()
        )
        
        return icon
    
    def _update_icon_loop(self):
        """Background thread to update icon and menu periodically."""
        while self.is_running:
            try:
                if self.icon and self.is_running:
                    # Update menu
                    new_menu = self.menu_manager.create_menu()
                    self.icon.menu = new_menu
                    
                    # Update icon if status changed
                    new_icon_image = self._get_current_icon()
                    self.icon.icon = new_icon_image
                    
                time.sleep(self.config.refresh_interval)
                
            except Exception as e:
                print(f"Error in update loop: {e}")
                time.sleep(1)
    
    def run(self):
        """Run the menu bar application."""
        try:
            print("🚀 Starting Portify Menu Bar App...")
            
            # Configure macOS settings (hide from dock)
            configure_macos_app()
            
            # Initial port scan
            self.menu_manager.refresh_ports()
            
            # Start menu manager
            self.menu_manager.start()
            
            # Create icon
            self.icon = self._create_icon()
            
            # Start update loop in background
            self.is_running = True
            update_thread = threading.Thread(target=self._update_icon_loop, daemon=True)
            update_thread.start()
            
            print("✅ Portify is now running in the menu bar!")
            print("💡 Look for the 'P' icon in your menu bar")
            print("🔄 The menu will auto-refresh every 5 seconds")
            print("⌨️  Press Ctrl+C to quit")
            
            # Run the icon (this blocks until quit)
            self.icon.run()
            
        except KeyboardInterrupt:
            print("\n👋 Shutting down Portify Menu Bar App...")
        except Exception as e:
            print(f"❌ Error running menu bar app: {e}")
            sys.exit(1)
        finally:
            self.stop()
    
    def stop(self):
        """Stop the application."""
        self.is_running = False
        if self.menu_manager:
            self.menu_manager.stop()
        if self.icon:
            self.icon.stop()


def run_menubar_app(
    max_ports: int = 7,
    refresh_interval: int = 5,
    notifications: bool = True,
    auto_refresh: bool = True
):
    """
    Run the Portify menu bar application.
    
    Args:
        max_ports: Maximum number of ports to show in menu
        refresh_interval: Refresh interval in seconds
        notifications: Whether to show notifications
        auto_refresh: Whether to auto-refresh
    """
    config = MenuConfig(
        max_ports_shown=max_ports,
        refresh_interval=refresh_interval,
        show_notifications=notifications,
        auto_refresh=auto_refresh
    )
    
    app = PortifyMenuBarApp(config)
    app.run()


if __name__ == "__main__":
    run_menubar_app()
