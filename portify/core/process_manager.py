"""
Process management functionality.
"""

import psutil
import signal
import os
from typing import Optional, Dict, Any
from enum import Enum


class KillResult(Enum):
    """Result of a kill operation."""
    SUCCESS = "success"
    NOT_FOUND = "not_found"
    ACCESS_DENIED = "access_denied"
    ALREADY_DEAD = "already_dead"
    ERROR = "error"


class ProcessManager:
    """Manager for process operations."""
    
    @staticmethod
    def kill_process(pid: int, force: bool = False) -> Dict[str, Any]:
        """
        Kill a process by PID.
        
        Args:
            pid: Process ID to kill
            force: Whether to use SIGKILL instead of SIGTERM
            
        Returns:
            Dictionary with result information
        """
        if pid <= 0:
            return {
                "result": KillResult.ERROR,
                "message": "Invalid PID",
                "pid": pid
            }
        
        try:
            process = psutil.Process(pid)
            process_name = process.name()
            
            # Check if process exists and is running
            if not process.is_running():
                return {
                    "result": KillResult.ALREADY_DEAD,
                    "message": f"Process {process_name} (PID: {pid}) is not running",
                    "pid": pid,
                    "process_name": process_name
                }
            
            # Try to terminate the process
            if force:
                process.kill()  # SIGKILL
                action = "killed (SIGKILL)"
            else:
                process.terminate()  # SIGTERM
                action = "terminated (SIGTERM)"
            
            # Wait for the process to actually die
            try:
                process.wait(timeout=3)
            except psutil.TimeoutExpired:
                if not force:
                    # If SIGTERM didn't work, try SIGKILL
                    process.kill()
                    action = "killed (SIGKILL after SIGTERM timeout)"
                    try:
                        process.wait(timeout=2)
                    except psutil.TimeoutExpired:
                        return {
                            "result": KillResult.ERROR,
                            "message": f"Process {process_name} (PID: {pid}) could not be killed",
                            "pid": pid,
                            "process_name": process_name
                        }
            
            return {
                "result": KillResult.SUCCESS,
                "message": f"Process {process_name} (PID: {pid}) {action}",
                "pid": pid,
                "process_name": process_name
            }
            
        except psutil.NoSuchProcess:
            return {
                "result": KillResult.NOT_FOUND,
                "message": f"Process with PID {pid} not found",
                "pid": pid
            }
            
        except psutil.AccessDenied:
            return {
                "result": KillResult.ACCESS_DENIED,
                "message": f"Access denied when trying to kill PID {pid}. Try running with sudo.",
                "pid": pid
            }
            
        except Exception as e:
            return {
                "result": KillResult.ERROR,
                "message": f"Error killing process {pid}: {str(e)}",
                "pid": pid
            }
    
    @staticmethod
    def get_process_info(pid: int) -> Optional[Dict[str, Any]]:
        """
        Get detailed information about a process.
        
        Args:
            pid: Process ID
            
        Returns:
            Dictionary with process information or None if not found
        """
        try:
            process = psutil.Process(pid)
            
            return {
                "pid": pid,
                "name": process.name(),
                "status": process.status(),
                "cpu_percent": process.cpu_percent(),
                "memory_mb": process.memory_info().rss / 1024 / 1024,
                "create_time": process.create_time(),
                "cmdline": " ".join(process.cmdline()) if process.cmdline() else "",
                "username": process.username() if hasattr(process, 'username') else None,
                "connections": len(process.connections()) if hasattr(process, 'connections') else 0
            }
            
        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            return None
    
    @staticmethod
    def is_process_running(pid: int) -> bool:
        """Check if a process is running."""
        try:
            process = psutil.Process(pid)
            return process.is_running()
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return False
    
    @staticmethod
    def get_processes_by_name(name: str) -> list:
        """Get all processes matching a name pattern."""
        matching_processes = []
        
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                if name.lower() in proc.info['name'].lower():
                    matching_processes.append(proc.info)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
                
        return matching_processes
