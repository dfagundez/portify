"""
Utility functions for Portify.
"""

import os
import sys
import platform
from typing import Dict, Any


def get_system_info() -> Dict[str, Any]:
    """Get basic system information."""
    return {
        "platform": platform.system(),
        "platform_version": platform.version(),
        "architecture": platform.architecture()[0],
        "python_version": sys.version.split()[0],
        "user": os.getenv("USER", "unknown"),
        "is_root": os.geteuid() == 0 if hasattr(os, 'geteuid') else False
    }


def format_bytes(bytes_value: float) -> str:
    """Format bytes into human readable format."""
    if bytes_value < 1024:
        return f"{bytes_value:.1f} B"
    elif bytes_value < 1024 * 1024:
        return f"{bytes_value / 1024:.1f} KB"
    elif bytes_value < 1024 * 1024 * 1024:
        return f"{bytes_value / (1024 * 1024):.1f} MB"
    else:
        return f"{bytes_value / (1024 * 1024 * 1024):.1f} GB"


def is_port_in_range(port: int, start: int = 1, end: int = 65535) -> bool:
    """Check if port is in valid range."""
    return start <= port <= end


def is_privileged_port(port: int) -> bool:
    """Check if port is privileged (< 1024)."""
    return port < 1024


def get_port_description(port: int) -> str:
    """Get common description for well-known ports."""
    common_ports = {
        20: "FTP Data",
        21: "FTP Control",
        22: "SSH",
        23: "Telnet",
        25: "SMTP",
        53: "DNS",
        80: "HTTP",
        110: "POP3",
        143: "IMAP",
        443: "HTTPS",
        993: "IMAPS",
        995: "POP3S",
        3000: "Development Server",
        3001: "Development Server",
        4000: "Development Server",
        5000: "Development Server",
        5432: "PostgreSQL",
        5672: "RabbitMQ",
        6379: "Redis",
        8000: "Development Server",
        8080: "HTTP Alternate",
        8443: "HTTPS Alternate",
        9000: "Development Server",
        27017: "MongoDB"
    }
    
    return common_ports.get(port, "Unknown")


def requires_root_warning() -> str:
    """Get warning message for operations that might require root."""
    return "⚠️  Some operations may require root privileges (sudo)"


def get_color_for_status(status: str) -> str:
    """Get color code for connection status."""
    status_colors = {
        "LISTEN": "green",
        "ESTABLISHED": "blue",
        "TIME_WAIT": "yellow",
        "CLOSE_WAIT": "orange",
        "FIN_WAIT1": "red",
        "FIN_WAIT2": "red",
        "CLOSING": "red",
        "CLOSED": "dim"
    }
    
    return status_colors.get(status, "white")
