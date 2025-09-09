"""
CLI commands for Portify using Typer.
"""

import typer
import time
from typing import Optional
from rich.console import Console

from ..core.port_scanner import PortScanner
from ..core.process_manager import ProcessManager
from ..core.utils import get_system_info, requires_root_warning
from .display import PortifyDisplay


app = typer.Typer(
    name="portify",
    help="🚀 Portify - Port and Process Manager for Developers",
    add_completion=False,
    rich_markup_mode="rich"
)

# Global display instance
display = PortifyDisplay()


@app.command("list")
def list_ports(
    system: bool = typer.Option(
        False, 
        "--system", 
        "-s", 
        help="Include CPU and memory usage information"
    ),
    filter_name: Optional[str] = typer.Option(
        None, 
        "--filter", 
        "-f", 
        help="Filter by process name"
    ),
    port: Optional[int] = typer.Option(
        None, 
        "--port", 
        "-p", 
        help="Filter by specific port number"
    ),
    listening: bool = typer.Option(
        False, 
        "--listening", 
        "-l", 
        help="Show only listening ports"
    )
):
    """📋 List all active ports and their processes."""
    
    try:
        display.show_loading("Scanning ports...")
        
        scanner = PortScanner()
        ports = scanner.scan_ports(include_system_info=system)
        
        # Apply filters
        if filter_name:
            ports = scanner.get_ports_by_process(filter_name)
            if not ports:
                display.show_warning(f"No processes found matching '{filter_name}'")
                return
        
        if port:
            ports = scanner.get_port_by_number(port)
            if not ports:
                display.show_warning(f"No processes found using port {port}")
                return
        
        if listening:
            ports = scanner.get_listening_ports()
            if not ports:
                display.show_info("No listening ports found")
                return
        
        display.show_ports_table(ports, show_system_info=system)
        
        if system:
            display.show_info("System information included (CPU/Memory usage)")
        
    except Exception as e:
        display.show_error("Failed to scan ports", e)
        raise typer.Exit(1)


@app.command("kill")
def kill_process(
    pid: int = typer.Argument(..., help="Process ID to kill"),
    force: bool = typer.Option(
        False, 
        "--force", 
        "-f", 
        help="Use SIGKILL instead of SIGTERM"
    ),
    yes: bool = typer.Option(
        False, 
        "--yes", 
        "-y", 
        help="Skip confirmation prompt"
    )
):
    """⚡ Kill a process by its PID."""
    
    if pid <= 0:
        display.show_error("Invalid PID. PID must be a positive integer.")
        raise typer.Exit(1)
    
    try:
        # Get process info first
        process_info = ProcessManager.get_process_info(pid)
        
        if not process_info:
            display.show_error(f"Process with PID {pid} not found or access denied")
            raise typer.Exit(1)
        
        # Ask for confirmation unless --yes is used
        if not yes:
            if not display.confirm_kill(pid, process_info["name"]):
                display.show_info("Operation cancelled")
                return
        
        # Kill the process
        result = ProcessManager.kill_process(pid, force=force)
        display.show_process_killed(result)
        
        # Show warning if access was denied
        from ..core.process_manager import KillResult
        if result["result"] == KillResult.ACCESS_DENIED:
            display.show_warning(requires_root_warning())
        
    except Exception as e:
        display.show_error(f"Failed to kill process {pid}", e)
        raise typer.Exit(1)


@app.command("monitor")
def monitor_ports(
    interval: int = typer.Option(
        2, 
        "--interval", 
        "-i", 
        help="Refresh interval in seconds"
    ),
    system: bool = typer.Option(
        False, 
        "--system", 
        "-s", 
        help="Include CPU and memory usage information"
    )
):
    """🔄 Monitor ports in real-time (interactive mode)."""
    
    display.show_info(f"Starting port monitor (refresh every {interval}s). Press Ctrl+C to stop.")
    display.show_info("Commands: 'q' to quit, 'r' to refresh, 'c' to clear screen")
    
    try:
        scanner = PortScanner()
        
        while True:
            try:
                display.clear_screen()
                display.show_banner()
                
                ports = scanner.scan_ports(include_system_info=system)
                display.show_ports_table(ports, show_system_info=system)
                
                display.print_separator()
                display.show_info(f"Last updated: {time.strftime('%H:%M:%S')} | Next refresh in {interval}s")
                
                time.sleep(interval)
                
            except KeyboardInterrupt:
                display.show_info("\nMonitoring stopped by user")
                break
                
    except Exception as e:
        display.show_error("Error during monitoring", e)
        raise typer.Exit(1)


@app.command("info")
def show_info():
    """ℹ️ Show system information."""
    
    try:
        system_info = get_system_info()
        display.show_system_info(system_info)
        
        if not system_info["is_root"]:
            display.show_warning(requires_root_warning())
            
    except Exception as e:
        display.show_error("Failed to get system information", e)
        raise typer.Exit(1)


@app.command("menubar")
def run_menubar(
    max_ports: int = typer.Option(
        7,
        "--max-ports",
        "-m",
        help="Maximum number of ports to show in menu"
    ),
    refresh_interval: int = typer.Option(
        5,
        "--interval",
        "-i",
        help="Refresh interval in seconds"
    ),
    no_notifications: bool = typer.Option(
        False,
        "--no-notifications",
        help="Disable system notifications"
    ),
    no_auto_refresh: bool = typer.Option(
        False,
        "--no-auto-refresh",
        help="Disable auto-refresh"
    )
):
    """🎯 Run Portify as a menu bar application."""
    
    try:
        # Import here to avoid issues if menubar dependencies aren't installed
        from ..menubar.app import run_menubar_app
        
        display.show_info("Starting Portify Menu Bar App...")
        
        run_menubar_app(
            max_ports=max_ports,
            refresh_interval=refresh_interval,
            notifications=not no_notifications,
            auto_refresh=not no_auto_refresh
        )
        
    except ImportError as e:
        display.show_error(
            "Menu bar dependencies not installed. Install with: pip install -r requirements-menubar.txt",
            e
        )
        raise typer.Exit(1)
    except Exception as e:
        display.show_error("Failed to start menu bar app", e)
        raise typer.Exit(1)


@app.command("version")
def show_version():
    """📦 Show Portify version."""
    
    from .. import __version__, __author__
    
    version_info = f"""
🚀 Portify v{__version__}
👨‍💻 Created by {__author__}
🐍 Python-powered port and process manager
    """.strip()
    
    display.console.print(version_info, style="blue")


@app.callback()
def main(
    version: bool = typer.Option(
        False, 
        "--version", 
        "-v", 
        help="Show version and exit"
    )
):
    """🚀 Portify - A lightweight CLI tool for developers to manage ports and processes."""
    
    if version:
        show_version()
        raise typer.Exit()


if __name__ == "__main__":
    app()
