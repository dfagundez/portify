"""
Menu management for Portify menu bar app.
"""

import threading
import time
from typing import List, Callable, Optional
from dataclasses import dataclass

import pystray
from pystray import MenuItem, Menu
from PIL import Image

from ..core.port_scanner import PortScanner, PortInfo
from ..core.process_manager import ProcessManager, KillResult
from .notifications import NotificationManager


@dataclass
class MenuConfig:
    """Configuration for the menu bar app."""
    max_ports_shown: int = 7
    refresh_interval: int = 5
    show_notifications: bool = True
    auto_refresh: bool = True


class MenuManager:
    """Manages the menu bar application and its menu items."""
    
    def __init__(self, config: Optional[MenuConfig] = None):
        self.config = config or MenuConfig()
        self.scanner = PortScanner()
        self.process_manager = ProcessManager()
        self.notification_manager = NotificationManager()
        self.current_ports: List[PortInfo] = []
        self.is_running = False
        self.refresh_thread: Optional[threading.Thread] = None
        
    def get_current_status(self) -> str:
        """Determine current system status based on ports."""
        if not self.current_ports:
            return 'inactive'
        
        listening_count = len([p for p in self.current_ports if p.status == 'LISTEN'])
        total_count = len(self.current_ports)
        
        if listening_count > 10:
            return 'warning'
        elif listening_count > 0:
            return 'active'
        elif total_count > 20:
            return 'warning'
        else:
            return 'normal'
    
    def refresh_ports(self) -> None:
        """Refresh the current ports list."""
        try:
            self.current_ports = self.scanner.scan_ports()
        except Exception as e:
            print(f"Error refreshing ports: {e}")
            self.current_ports = []
    
    def kill_process_with_confirmation(self, pid: int, process_name: str) -> Callable:
        """Create a callback function to kill a process with confirmation."""
        def kill_callback(icon, item):
            try:
                # Show notification before killing
                if self.config.show_notifications:
                    self._show_notification(
                        "Portify", 
                        f"Killing process {process_name} (PID: {pid})"
                    )
                
                result = self.process_manager.kill_process(pid)
                
                # Show result notification
                if self.config.show_notifications:
                    if result["result"] == KillResult.SUCCESS:
                        self._show_notification(
                            "Portify - Success", 
                            f"Process {process_name} killed successfully"
                        )
                    else:
                        self._show_notification(
                            "Portify - Error", 
                            result["message"]
                        )
                
                # Refresh menu after killing
                self.refresh_ports()
                
            except Exception as e:
                if self.config.show_notifications:
                    self._show_notification(
                        "Portify - Error", 
                        f"Failed to kill process: {str(e)}"
                    )
        
        return kill_callback
    
    def _show_notification(self, title: str, message: str) -> None:
        """Show system notification."""
        if self.config.show_notifications:
            self.notification_manager.show_notification(title, message)
    
    def create_port_menu_item(self, port: PortInfo) -> MenuItem:
        """Create a menu item for a specific port."""
        # Format the display text
        status_emoji = {
            'LISTEN': '🟢',
            'ESTABLISHED': '🔵', 
            'TIME_WAIT': '🟡',
            'CLOSE_WAIT': '🟠',
            'CLOSED': '⚫'
        }.get(port.status, '⚪')
        
        # Truncate process name if too long
        process_name = port.process_name
        if len(process_name) > 15:
            process_name = process_name[:12] + "..."
        
        display_text = f"{status_emoji} {process_name} ({port.port})"
        
        # Create submenu for this port
        submenu = Menu(
            MenuItem(
                f"Kill Process (PID: {port.pid})",
                self.kill_process_with_confirmation(port.pid, port.process_name),
                enabled=port.pid > 0
            ),
            MenuItem(f"Protocol: {port.protocol}", None, enabled=False),
            MenuItem(f"Status: {port.status}", None, enabled=False),
            MenuItem(f"Address: {port.local_address}", None, enabled=False)
        )
        
        return MenuItem(display_text, submenu)
    
    def create_menu(self) -> Menu:
        """Create the complete menu for the menu bar."""
        menu_items = []
        
        # Header
        port_count = len(self.current_ports)
        listening_count = len([p for p in self.current_ports if p.status == 'LISTEN'])
        
        menu_items.append(
            MenuItem(
                f"🚀 Portify ({port_count} connections)", 
                None, 
                enabled=False
            )
        )
        
        menu_items.append(MenuItem("─" * 25, None, enabled=False))
        
        if not self.current_ports:
            menu_items.append(MenuItem("📭 No active ports", None, enabled=False))
        else:
            # Show summary
            menu_items.append(
                MenuItem(
                    f"📊 {listening_count} listening, {port_count} total",
                    None,
                    enabled=False
                )
            )
            
            menu_items.append(MenuItem("─" * 25, None, enabled=False))
            
            # Show top ports (listening first, then others)
            listening_ports = [p for p in self.current_ports if p.status == 'LISTEN']
            other_ports = [p for p in self.current_ports if p.status != 'LISTEN']
            
            # Sort by port number
            listening_ports.sort(key=lambda x: x.port)
            other_ports.sort(key=lambda x: x.port)
            
            # Show listening ports first
            shown_count = 0
            for port in listening_ports:
                if shown_count >= self.config.max_ports_shown:
                    break
                menu_items.append(self.create_port_menu_item(port))
                shown_count += 1
            
            # Show other ports if we have space
            for port in other_ports:
                if shown_count >= self.config.max_ports_shown:
                    break
                menu_items.append(self.create_port_menu_item(port))
                shown_count += 1
            
            # Show "more" indicator if there are more ports
            if len(self.current_ports) > self.config.max_ports_shown:
                remaining = len(self.current_ports) - self.config.max_ports_shown
                menu_items.append(
                    MenuItem(
                        f"... and {remaining} more",
                        None,
                        enabled=False
                    )
                )
        
        # Separator
        menu_items.append(MenuItem("─" * 25, None, enabled=False))
        
        # Actions
        menu_items.append(
            MenuItem(
                "🔄 Refresh Now",
                lambda icon, item: self.refresh_ports()
            )
        )
        
        menu_items.append(
            MenuItem(
                "📋 Open CLI",
                lambda icon, item: self._open_cli()
            )
        )
        
        menu_items.append(
            MenuItem(
                f"⚙️ Auto-refresh: {'ON' if self.config.auto_refresh else 'OFF'}",
                lambda icon, item: self._toggle_auto_refresh()
            )
        )
        
        # Separator
        menu_items.append(MenuItem("─" * 25, None, enabled=False))
        
        # Quit
        menu_items.append(
            MenuItem(
                "🚪 Quit Portify",
                lambda icon, item: self._quit_app(icon)
            )
        )
        
        return Menu(*menu_items)
    
    def _open_cli(self) -> None:
        """Open terminal with portify list command."""
        import subprocess
        import os
        
        try:
            # Open Terminal and run portify list
            script = 'tell application "Terminal" to do script "portify list"'
            subprocess.run(['osascript', '-e', script])
        except Exception as e:
            print(f"Failed to open CLI: {e}")
    
    def _toggle_auto_refresh(self) -> None:
        """Toggle auto-refresh functionality."""
        self.config.auto_refresh = not self.config.auto_refresh
        if self.config.auto_refresh and not self.refresh_thread:
            self._start_refresh_thread()
        elif not self.config.auto_refresh and self.refresh_thread:
            self._stop_refresh_thread()
    
    def _quit_app(self, icon) -> None:
        """Quit the application."""
        self.is_running = False
        if self.refresh_thread:
            self._stop_refresh_thread()
        icon.stop()
    
    def _start_refresh_thread(self) -> None:
        """Start the auto-refresh thread."""
        if self.refresh_thread and self.refresh_thread.is_alive():
            return
            
        self.refresh_thread = threading.Thread(target=self._refresh_loop, daemon=True)
        self.refresh_thread.start()
    
    def _stop_refresh_thread(self) -> None:
        """Stop the auto-refresh thread."""
        self.is_running = False
        if self.refresh_thread:
            self.refresh_thread.join(timeout=1)
    
    def _refresh_loop(self) -> None:
        """Auto-refresh loop that runs in a separate thread."""
        while self.is_running and self.config.auto_refresh:
            time.sleep(self.config.refresh_interval)
            if self.is_running:
                self.refresh_ports()
    
    def start(self) -> None:
        """Start the menu manager."""
        self.is_running = True
        if self.config.auto_refresh:
            self._start_refresh_thread()
    
    def stop(self) -> None:
        """Stop the menu manager."""
        self.is_running = False
        self._stop_refresh_thread()
