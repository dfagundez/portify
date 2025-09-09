"""
Auto-updater functionality for Portify.
"""

import json
import urllib.request
import urllib.error
from typing import Dict, Optional, Tuple
from packaging import version
from .. import __version__


class UpdateChecker:
    """Checks for updates and manages version information."""
    
    def __init__(self, update_url: str = "https://tu-sitio-web.com/portify/version.json"):
        self.update_url = update_url
        self.current_version = __version__
    
    def check_for_updates(self) -> Dict[str, any]:
        """
        Check if there's a newer version available.
        
        Returns:
            Dict with update information
        """
        try:
            # Fetch version info from your server
            with urllib.request.urlopen(self.update_url, timeout=5) as response:
                data = json.loads(response.read().decode())
            
            latest_version = data.get("version", "0.0.0")
            download_url = data.get("download_url", "")
            release_notes = data.get("release_notes", "")
            critical = data.get("critical", False)
            
            # Compare versions
            if version.parse(latest_version) > version.parse(self.current_version):
                return {
                    "update_available": True,
                    "current_version": self.current_version,
                    "latest_version": latest_version,
                    "download_url": download_url,
                    "release_notes": release_notes,
                    "critical": critical
                }
            else:
                return {
                    "update_available": False,
                    "current_version": self.current_version,
                    "latest_version": latest_version
                }
                
        except (urllib.error.URLError, json.JSONDecodeError, KeyError) as e:
            return {
                "update_available": False,
                "error": f"Could not check for updates: {str(e)}",
                "current_version": self.current_version
            }
    
    def get_update_notification_text(self, update_info: Dict) -> str:
        """Generate notification text for updates."""
        if not update_info.get("update_available"):
            return ""
        
        latest = update_info["latest_version"]
        current = update_info["current_version"]
        critical = update_info.get("critical", False)
        
        if critical:
            return f"🚨 Critical Update Available: v{latest} (you have v{current})"
        else:
            return f"🆕 Update Available: v{latest} (you have v{current})"
