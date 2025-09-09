"""
Port scanning functionality using psutil.
"""

import psutil
import socket
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class PortInfo:
    """Information about a port and its associated process."""
    pid: int
    process_name: str
    port: int
    protocol: str
    status: str
    local_address: str
    remote_address: Optional[str] = None
    cpu_percent: Optional[float] = None
    memory_mb: Optional[float] = None


class PortScanner:
    """Scanner for active ports and their associated processes."""
    
    def __init__(self):
        self.ports_info: List[PortInfo] = []
    
    def scan_ports(self, include_system_info: bool = False) -> List[PortInfo]:
        """
        Scan all active network connections and return port information.
        
        Args:
            include_system_info: Whether to include CPU and memory usage
            
        Returns:
            List of PortInfo objects
        """
        ports_info = []
        
        try:
            # Get all network connections
            # On macOS, this might require special permissions
            try:
                connections = psutil.net_connections(kind='inet')
                connection_data = [(conn, conn.pid) for conn in connections]
            except psutil.AccessDenied:
                # Fallback: try to get connections from individual processes
                connection_data = []
                for proc in psutil.process_iter(['pid', 'name']):
                    try:
                        proc_connections = proc.connections(kind='inet')
                        for conn in proc_connections:
                            connection_data.append((conn, proc.pid))
                    except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                        continue
            
            for conn, conn_pid in connection_data:
                if conn.laddr and conn.laddr.port:
                    try:
                        # Get process information
                        if conn_pid:
                            try:
                                process = psutil.Process(conn_pid)
                                process_name = process.name()
                                
                                # Get system info if requested
                                cpu_percent = None
                                memory_mb = None
                                if include_system_info:
                                    try:
                                        cpu_percent = process.cpu_percent()
                                        memory_info = process.memory_info()
                                        memory_mb = memory_info.rss / 1024 / 1024  # Convert to MB
                                    except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                                        pass
                            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                                process_name = f"PID-{conn_pid}"
                                cpu_percent = None
                                memory_mb = None
                        else:
                            process_name = "System"
                            cpu_percent = None
                            memory_mb = None
                        
                        # Format addresses
                        local_addr = f"{conn.laddr.ip}:{conn.laddr.port}"
                        remote_addr = None
                        if conn.raddr:
                            remote_addr = f"{conn.raddr.ip}:{conn.raddr.port}"
                        
                        # Determine protocol
                        protocol = "TCP" if conn.type == socket.SOCK_STREAM else "UDP"
                        
                        port_info = PortInfo(
                            pid=conn_pid or 0,
                            process_name=process_name,
                            port=conn.laddr.port,
                            protocol=protocol,
                            status=conn.status,
                            local_address=local_addr,
                            remote_address=remote_addr,
                            cpu_percent=cpu_percent,
                            memory_mb=memory_mb
                        )
                        
                        ports_info.append(port_info)
                        
                    except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                        # Process might have died or we don't have permission
                        continue
                        
        except Exception as e:
            raise RuntimeError(f"Error scanning ports: {e}")
        
        # Sort by port number
        ports_info.sort(key=lambda x: x.port)
        self.ports_info = ports_info
        
        return ports_info
    
    def get_ports_by_process(self, process_name: str) -> List[PortInfo]:
        """Filter ports by process name."""
        return [port for port in self.ports_info 
                if process_name.lower() in port.process_name.lower()]
    
    def get_port_by_number(self, port_number: int) -> List[PortInfo]:
        """Get all processes using a specific port."""
        return [port for port in self.ports_info if port.port == port_number]
    
    def get_listening_ports(self) -> List[PortInfo]:
        """Get only listening ports."""
        return [port for port in self.ports_info 
                if port.status == psutil.CONN_LISTEN]
