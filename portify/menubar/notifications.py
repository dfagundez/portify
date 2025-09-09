"""
Cross-platform notification system for Portify.
"""

import sys
from typing import Optional


class NotificationManager:
    """Manages system notifications across platforms."""
    
    def __init__(self):
        self.available_methods = self._detect_available_methods()
    
    def _detect_available_methods(self) -> list:
        """Detect available notification methods."""
        methods = []
        
        if sys.platform == 'darwin':  # macOS
            try:
                import objc
                methods.append('macos_native')
            except ImportError:
                pass
            
            try:
                from plyer import notification
                methods.append('plyer')
            except ImportError:
                pass
        
        elif sys.platform.startswith('linux'):  # Linux
            try:
                from plyer import notification
                methods.append('plyer')
            except ImportError:
                pass
        
        methods.append('console')  # Always available fallback
        return methods
    
    def show_notification(self, title: str, message: str, timeout: int = 3) -> bool:
        """
        Show a system notification.
        
        Args:
            title: Notification title
            message: Notification message
            timeout: Timeout in seconds
            
        Returns:
            True if notification was shown successfully
        """
        for method in self.available_methods:
            try:
                if method == 'macos_native':
                    return self._show_macos_native(title, message)
                elif method == 'plyer':
                    return self._show_plyer(title, message, timeout)
                elif method == 'console':
                    return self._show_console(title, message)
            except Exception as e:
                print(f"Notification method {method} failed: {e}")
                continue
        
        return False
    
    def _show_macos_native(self, title: str, message: str) -> bool:
        """Show notification using native macOS APIs."""
        try:
            import objc
            from Foundation import NSUserNotification, NSUserNotificationCenter
            
            notification = NSUserNotification.alloc().init()
            notification.setTitle_(title)
            notification.setInformativeText_(message)
            notification.setSoundName_("NSUserNotificationDefaultSoundName")
            
            center = NSUserNotificationCenter.defaultUserNotificationCenter()
            center.deliverNotification_(notification)
            
            return True
        except Exception as e:
            print(f"macOS native notification failed: {e}")
            return False
    
    def _show_plyer(self, title: str, message: str, timeout: int) -> bool:
        """Show notification using plyer."""
        try:
            from plyer import notification
            notification.notify(
                title=title,
                message=message,
                timeout=timeout,
                app_name="Portify"
            )
            return True
        except Exception as e:
            print(f"Plyer notification failed: {e}")
            return False
    
    def _show_console(self, title: str, message: str) -> bool:
        """Fallback: show notification in console."""
        print(f"📢 {title}: {message}")
        return True
    
    def is_available(self) -> bool:
        """Check if any notification method is available."""
        return len(self.available_methods) > 1  # More than just console
    
    def get_status(self) -> dict:
        """Get notification system status."""
        return {
            "available_methods": self.available_methods,
            "primary_method": self.available_methods[0] if self.available_methods else None,
            "is_available": self.is_available()
        }
