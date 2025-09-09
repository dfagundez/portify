"""
Display formatting using Rich library.
"""

from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.text import Text
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.prompt import Confirm
from rich.live import Live
from typing import List, Optional
import time

from ..core.port_scanner import PortInfo
from ..core.utils import get_color_for_status, get_port_description, format_bytes


class PortifyDisplay:
    """Rich-based display manager for Portify."""
    
    def __init__(self):
        self.console = Console()
    
    def show_banner(self):
        """Display the Portify banner."""
        banner = Text("🚀 PORTIFY", style="bold blue")
        subtitle = Text("Port and Process Manager for Developers", style="dim")
        
        panel = Panel.fit(
            f"{banner}\n{subtitle}",
            border_style="blue",
            padding=(1, 2)
        )
        
        self.console.print(panel)
        self.console.print()
    
    def show_ports_table(self, ports: List[PortInfo], show_system_info: bool = False):
        """Display ports in a formatted table."""
        if not ports:
            self.console.print("📭 No active ports found", style="yellow")
            return
        
        table = Table(title="🌐 Active Ports", show_header=True, header_style="bold blue")
        
        # Add columns
        table.add_column("PID", style="cyan", width=8)
        table.add_column("Process", style="green", width=20)
        table.add_column("Port", style="magenta", width=8)
        table.add_column("Protocol", style="blue", width=8)
        table.add_column("Status", width=12)
        table.add_column("Local Address", style="dim", width=20)
        table.add_column("Description", style="dim", width=15)
        
        if show_system_info:
            table.add_column("CPU %", style="yellow", width=8)
            table.add_column("Memory", style="red", width=10)
        
        # Add rows
        for port in ports:
            # Format status with color
            status_color = get_color_for_status(port.status)
            status_text = Text(port.status, style=status_color)
            
            # Format PID
            pid_text = str(port.pid) if port.pid > 0 else "System"
            
            # Get port description
            description = get_port_description(port.port)
            
            row_data = [
                pid_text,
                port.process_name[:18] + "..." if len(port.process_name) > 18 else port.process_name,
                str(port.port),
                port.protocol,
                status_text,
                port.local_address,
                description
            ]
            
            if show_system_info:
                cpu_text = f"{port.cpu_percent:.1f}%" if port.cpu_percent is not None else "N/A"
                memory_text = f"{port.memory_mb:.1f}MB" if port.memory_mb is not None else "N/A"
                row_data.extend([cpu_text, memory_text])
            
            table.add_row(*row_data)
        
        self.console.print(table)
        self.console.print(f"\n📊 Total: {len(ports)} active connections")
    
    def show_process_killed(self, result: dict):
        """Display result of killing a process."""
        from ..core.process_manager import KillResult
        
        result_type = result["result"]
        message = result["message"]
        
        if result_type == KillResult.SUCCESS:
            self.console.print(f"✅ {message}", style="green")
        elif result_type == KillResult.NOT_FOUND:
            self.console.print(f"❌ {message}", style="red")
        elif result_type == KillResult.ACCESS_DENIED:
            self.console.print(f"🔒 {message}", style="yellow")
        elif result_type == KillResult.ALREADY_DEAD:
            self.console.print(f"💀 {message}", style="dim")
        else:
            self.console.print(f"⚠️ {message}", style="red")
    
    def show_error(self, message: str, exception: Optional[Exception] = None):
        """Display an error message."""
        error_text = f"❌ Error: {message}"
        if exception:
            error_text += f"\n   Details: {str(exception)}"
        
        panel = Panel(
            error_text,
            title="Error",
            border_style="red",
            padding=(1, 2)
        )
        
        self.console.print(panel)
    
    def show_warning(self, message: str):
        """Display a warning message."""
        self.console.print(f"⚠️ Warning: {message}", style="yellow")
    
    def show_info(self, message: str):
        """Display an info message."""
        self.console.print(f"ℹ️ {message}", style="blue")
    
    def show_success(self, message: str):
        """Display a success message."""
        self.console.print(f"✅ {message}", style="green")
    
    def confirm_kill(self, pid: int, process_name: str) -> bool:
        """Ask for confirmation before killing a process."""
        return Confirm.ask(
            f"Are you sure you want to kill process '{process_name}' (PID: {pid})?",
            default=False
        )
    
    def show_loading(self, message: str = "Scanning ports..."):
        """Show a loading spinner."""
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=self.console,
            transient=True,
        ) as progress:
            task = progress.add_task(description=message, total=None)
            time.sleep(0.5)  # Brief pause for visual feedback
    
    def show_system_info(self, system_info: dict):
        """Display system information."""
        info_text = f"""
Platform: {system_info['platform']} ({system_info['architecture']})
User: {system_info['user']} {'(root)' if system_info['is_root'] else '(user)'}
Python: {system_info['python_version']}
        """.strip()
        
        panel = Panel(
            info_text,
            title="System Information",
            border_style="blue",
            padding=(1, 2)
        )
        
        self.console.print(panel)
    
    def show_help_commands(self):
        """Display available commands."""
        help_text = """
[bold blue]Available Commands:[/bold blue]

[green]portify list[/green]              - List all active ports
[green]portify list --system[/green]     - List ports with CPU/Memory info
[green]portify kill <PID>[/green]        - Kill process by PID
[green]portify kill <PID> --force[/green] - Force kill process (SIGKILL)
[green]portify monitor[/green]           - Interactive monitoring mode
[green]portify info[/green]              - Show system information

[bold blue]Examples:[/bold blue]
  portify list
  portify kill 1234
  portify list --filter node
        """
        
        panel = Panel(
            help_text,
            title="Portify Help",
            border_style="blue",
            padding=(1, 2)
        )
        
        self.console.print(panel)
    
    def clear_screen(self):
        """Clear the console screen."""
        self.console.clear()
    
    def print_separator(self):
        """Print a visual separator."""
        self.console.print("─" * self.console.size.width, style="dim")
