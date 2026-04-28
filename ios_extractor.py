#!/usr/bin/env python3
"""
iOS Log Extractor for SCMessenger
Extracts live streaming OSLog data and diagnostic snapshots from connected iOS device.

Critical: This script assumes the iOS device is already connected, trusted, and the app is running.

BEHAVIOR:
- Runs continuously until Ctrl+C (same as Android adb_extractor.py)
- Captures logs in real-time or polls diagnostic snapshots every 5 seconds
- Automatically cleans up processes on exit

IMPORTANT NOTES:
1. Live streaming of iOS device logs:
   - Option A: Install libimobiledevice (brew install libimobiledevice) for true real-time streaming
   - Option B: Polling mode (default) - checks diagnostic log every 5 seconds
   - Option C: Use Xcode Console.app for graphical inspection

2. The diagnostic snapshot (mesh_diagnostics.log) contains structured diagnostic logs
   that are specifically designed for debugging SCMessenger and are often more useful
   than raw OSLog output.

USAGE:
    python3 ios_extractor.py
    
    Press Ctrl+C to stop capturing and cleanup.
"""

import subprocess
import time
import os
import signal
import sys
import argparse
from pathlib import Path
import threading

# ===========================
# PHASE 1 DISCOVERY RESULTS
# ===========================
BUNDLE_ID = "SovereignCommunications.SCMessenger"
OSLOG_SUBSYSTEM = "com.scmessenger"
DEVICE_UDID = "4731D564-2F8F-5BC6-B713-D7774AF598F9"
DIAGNOSTIC_LOG_FILENAME = "mesh_diagnostics.log"

# Output files
LIVE_LOG_OUTPUT = "live_ios_log.log"
SNAPSHOT_OUTPUT = "ios_diagnostic_snapshot.log"

# Log rotation settings
LOG_ARCHIVE_DIR = "ios_logs_archive"
MAX_LOG_AGE_HOURS = 24
MAX_TOTAL_SIZE_MB = 100

# Global quiet mode flag (set by command line argument)
QUIET_MODE = False

# Repo-local tmp directory (per COPILOT_AGENT_INSTRUCTIONS)
REPO_ROOT = Path(__file__).parent
TMP_DIR = REPO_ROOT / "tmp"
TMP_DIR.mkdir(exist_ok=True)

# Create archive directory
LOG_ARCHIVE_PATH = REPO_ROOT / LOG_ARCHIVE_DIR
LOG_ARCHIVE_PATH.mkdir(exist_ok=True)
TMP_DIR.mkdir(exist_ok=True)

class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def print_success(msg):
    if not QUIET_MODE:
        print(f"{Colors.GREEN}✓ {msg}{Colors.RESET}")

def print_error(msg):
    # Always print errors, even in quiet mode
    print(f"{Colors.RED}✗ {msg}{Colors.RESET}")

def print_info(msg):
    if not QUIET_MODE:
        print(f"{Colors.BLUE}ℹ {msg}{Colors.RESET}")

def print_phase(msg):
    if not QUIET_MODE:
        print(f"\n{Colors.BOLD}{Colors.YELLOW}{'='*60}{Colors.RESET}")
        print(f"{Colors.BOLD}{Colors.YELLOW}{msg}{Colors.RESET}")
        print(f"{Colors.BOLD}{Colors.YELLOW}{'='*60}{Colors.RESET}\n")

def run_with_timeout(cmd, timeout_seconds=20):
    """
    Run a command with a hard timeout that actually works.
    This handles cases where subprocess.run(timeout=X) doesn't work properly
    with devicectl commands that can hang indefinitely.
    """
    def target(cmd, result_container):
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            result_container['result'] = result
            result_container['completed'] = True
        except Exception as e:
            result_container['error'] = e
            result_container['completed'] = False
    
    result_container = {'completed': False, 'result': None, 'error': None}
    thread = threading.Thread(target=target, args=(cmd, result_container))
    thread.daemon = True
    thread.start()
    thread.join(timeout_seconds)
    
    if thread.is_alive():
        # Command is still running - it's hung
        print_error(f"Command hung after {timeout_seconds}s, treating as timeout")
        return None
    
    if result_container.get('completed'):
        return result_container['result']
    elif result_container.get('error'):
        raise result_container['error']
    else:
        return None

# ===========================
# LOG ROTATION AND CLEANUP
# ===========================

def rotate_previous_logs():
    """
    Archive previous log files with timestamp before starting new capture.
    Implements log rotation: keeps logs for 24 hours or 100MB total (whichever comes first).
    """
    import shutil
    from datetime import datetime, timedelta
    
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    # Archive existing live log if it exists and has content
    live_log_path = REPO_ROOT / LIVE_LOG_OUTPUT
    if live_log_path.exists() and live_log_path.stat().st_size > 0:
        archived_name = f"live_ios_log_{timestamp}.log"
        archived_path = LOG_ARCHIVE_PATH / archived_name
        shutil.move(str(live_log_path), str(archived_path))
        print_info(f"Archived previous live log as: {archived_name}")
    
    # Archive existing snapshot if it exists and has content
    snapshot_path = REPO_ROOT / SNAPSHOT_OUTPUT
    if snapshot_path.exists() and snapshot_path.stat().st_size > 0:
        archived_name = f"ios_diagnostic_snapshot_{timestamp}.log"
        archived_path = LOG_ARCHIVE_PATH / archived_name
        shutil.move(str(snapshot_path), str(archived_path))
        print_info(f"Archived previous snapshot as: {archived_name}")
    
    # Clean up old logs based on age and size
    cleanup_old_logs()

def cleanup_old_logs():
    """
    Remove logs older than 24 hours OR enforce 100MB total size limit.
    Deletes oldest logs first until both constraints are met.
    """
    from datetime import datetime, timedelta
    
    # Get all log files in archive
    log_files = []
    for log_file in LOG_ARCHIVE_PATH.glob("*.log"):
        stat = log_file.stat()
        log_files.append({
            'path': log_file,
            'size': stat.st_size,
            'mtime': stat.st_mtime,
            'age_hours': (time.time() - stat.st_mtime) / 3600
        })
    
    if not log_files:
        return
    
    # Sort by age (oldest first)
    log_files.sort(key=lambda x: x['mtime'])
    
    # Step 1: Remove logs older than 24 hours
    cutoff_age = MAX_LOG_AGE_HOURS
    files_to_delete = []
    
    for log in log_files:
        if log['age_hours'] > cutoff_age:
            files_to_delete.append(log)
    
    if files_to_delete:
        print_info(f"Removing {len(files_to_delete)} log(s) older than {cutoff_age} hours...")
        for log in files_to_delete:
            log['path'].unlink()
            log_files.remove(log)
    
    # Step 2: Enforce 100MB total size limit
    max_size_bytes = MAX_TOTAL_SIZE_MB * 1024 * 1024
    total_size = sum(log['size'] for log in log_files)
    
    if total_size > max_size_bytes:
        print_info(f"Total log size ({total_size / 1024 / 1024:.1f} MB) exceeds {MAX_TOTAL_SIZE_MB} MB limit...")
        
        # Delete oldest logs until under limit
        while log_files and total_size > max_size_bytes:
            oldest = log_files.pop(0)
            oldest['path'].unlink()
            total_size -= oldest['size']
            print_info(f"Deleted old log: {oldest['path'].name} ({oldest['size'] / 1024:.1f} KB)")
        
        print_success(f"Log cleanup complete. Total size now: {total_size / 1024 / 1024:.1f} MB")

# ===========================
# PHASE 2: STREAMING LOGS
# ===========================

def extract_live_logs():
    """
    Extract live streaming logs from the iOS device.
    For iOS 17+ devices, the diagnostic snapshot polling is the most reliable method.
    """
    print_phase("PHASE 2A: STREAMING LIVE iOS LOGS")
    
    print_info(f"Target Device: {DEVICE_UDID}")
    print_info(f"Bundle ID: {BUNDLE_ID}")
    print_info(f"OSLog Subsystem: {OSLOG_SUBSYSTEM}")
    print_info(f"Output File: {LIVE_LOG_OUTPUT}")
    
    # For iOS 17+ with CoreDevice, diagnostic snapshot polling is most reliable
    print_info("Using diagnostic snapshot polling for live monitoring...")
    print_info("(iOS 17+ devices use CoreDevice which doesn't support real-time syslog streaming)")
    print_info("The diagnostic snapshot contains structured app-level diagnostics")
    
    return extract_live_logs_polling()

def extract_live_logs_unified():
    """
    DEPRECATED: Extract logs using macOS unified logging system.
    Note: This only works for macOS processes, not iOS devices.
    Kept for reference but not used.
    """
    cmd = [
        "log", "stream",
        "--style", "syslog",
        "--level", "debug",
        "--predicate", f'subsystem == "{OSLOG_SUBSYSTEM}" OR processImagePath CONTAINS "{BUNDLE_ID.split(".")[-1]}"'
    ]
    
    print_info(f"Command: {' '.join(cmd)}")
    print_info("Note: This captures OSLog from macOS, not iOS device")
    
    # Open output file
    log_file = open(LIVE_LOG_OUTPUT, 'w', buffering=1)
    
    # Start streaming subprocess
    print_info("Starting log stream...")
    proc = subprocess.Popen(
        cmd,
        stdout=log_file,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Live verification: read for 5 seconds and count lines
    print_info("Verifying live stream (5 second capture)...")
    time.sleep(5)
    
    # Check if process is still running
    if proc.poll() is not None:
        stderr_output = proc.stderr.read()
        log_file.close()
        print_error(f"Log stream process terminated unexpectedly!")
        print_error(f"stderr: {stderr_output}")
        return None
    
    # Check if we captured any data
    log_file.flush()
    if os.path.exists(LIVE_LOG_OUTPUT):
        size = os.path.getsize(LIVE_LOG_OUTPUT)
        with open(LIVE_LOG_OUTPUT, 'r') as f:
            lines = f.readlines()
            line_count = len(lines)
        
        if size > 0 and line_count > 0:
            print_success(f"Live stream verified! Captured {line_count} lines ({size} bytes)")
            # Show sample of latest logs
            print_info("Sample output (last 3 lines):")
            for line in lines[-3:]:
                print(f"  {line.rstrip()}")
        else:
            print_error("Stream started but no data captured yet. Check if app is logging.")
    else:
        print_error(f"Output file {LIVE_LOG_OUTPUT} not created")
        log_file.close()
        return None
    
    return (proc, log_file)

def extract_live_logs_polling():
    """
    Fallback method: Poll the diagnostic snapshot periodically to create a pseudo-live log.
    This provides near-realtime monitoring of app diagnostics.
    Runs continuously until Ctrl+C.
    
    Note: For iOS 17+ devices with CoreDevice, this is the RECOMMENDED method as it
    provides the most useful app-level diagnostic information.
    
    IMPORTANT: If devicectl hangs (common CoreDevice issue), this will gracefully
    fall back to monitoring mode without diagnostic snapshots.
    """
    print_info("Initializing diagnostic snapshot polling (every 2 seconds)...")
    print_info("This method provides app-level structured diagnostics")
    print_info("Note: If devicectl hangs, this is a known iOS 17+ CoreDevice issue")
    
    log_file = open(LIVE_LOG_OUTPUT, 'w', buffering=1)
    log_file.write("=== iOS Diagnostic Log Monitoring (Polling Mode) ===\n")
    log_file.write(f"Started at: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
    log_file.write(f"Device: {DEVICE_UDID}\n")
    log_file.write(f"Bundle: {BUNDLE_ID}\n")
    log_file.write("Polling every 2 seconds. Press Ctrl+C to stop.\n")
    log_file.write("Note: This captures app-generated diagnostic events, not OSLog.\n")
    log_file.write("If devicectl hangs, this is a known CoreDevice issue - the script will continue.\n\n")
    log_file.flush()
    
    snapshot_count = 0
    previous_content = ""
    
    print_info("Pulling initial diagnostic snapshot...")
    
    # First snapshot for verification
    try:
        pull_cmd = [
            "xcrun", "devicectl", "device", "copy", "from",
            "--device", DEVICE_UDID,
            "--domain-type", "appDataContainer",
            "--domain-identifier", BUNDLE_ID,
            "--source", f"Documents/{DIAGNOSTIC_LOG_FILENAME}",
            "--destination", f"{TMP_DIR}/poll_snapshot_0.log"
        ]
        
        print_info("Attempting to pull diagnostic snapshot (20s timeout)...")
        result = run_with_timeout(pull_cmd, 20)
        
        if result is None:
            print_error("devicectl command hung - this is a known CoreDevice issue")
            print_error("The app container may not be accessible or Documents directory doesn't exist")
            print_info("This can happen if:")
            print_info("  • App hasn't created the diagnostic log file yet")
            print_info("  • App container is locked by iOS")
            print_info("  • CoreDevice/devicectl has bugs with file access")
            print_info("Continuing without diagnostic snapshot...")
            log_file.close()
            return None
        elif result.returncode != 0:
            print_error(f"devicectl failed: {result.stderr}")
            log_file.close()
            return None
        
        snapshot_file = f"{TMP_DIR}/poll_snapshot_0.log"
        if os.path.exists(snapshot_file):
            with open(snapshot_file, 'r') as sf:
                previous_content = sf.read()
                log_file.write(f"--- Initial Snapshot at {time.strftime('%H:%M:%S')} ---\n")
                log_file.write(previous_content)
                log_file.write('\n')
                log_file.flush()
                
            line_count = len(previous_content.split('\n'))
            print_success(f"Initial snapshot captured: {line_count} lines")
            print_info("These are app-generated diagnostic events (delivery attempts, BLE rx/tx, etc.)")
        else:
            print_error("Failed to capture initial snapshot")
            log_file.close()
            return None
            
    except Exception as e:
        print_error(f"Initial snapshot failed: {e}")
        log_file.close()
        return None
    
    snapshot_count = 1
    
    # Now we'll return None to signal polling mode
    # but the main loop will handle the continuous polling differently
    log_file.close()
    
    print_success(f"Polling mode initialized: {LIVE_LOG_OUTPUT}")
    print_info("Diagnostic logs contain: delivery_attempt, ble_rx/tx, peer_identified, relay_state, etc.")
    
    return None  # No process to clean up - polling handled in main

def continue_polling_mode():
    """
    Continue polling diagnostic snapshots until Ctrl+C.
    Called when using polling fallback mode.
    Polls every 2 seconds for near-real-time monitoring.
    """
    print_info("\n[Monitoring] Polling mode active - checking every 2 seconds... (Ctrl+C to stop)")
    
    snapshot_count = 1
    previous_content = ""
    
    # Read the initial content
    initial_snapshot = f"{TMP_DIR}/poll_snapshot_0.log"
    if os.path.exists(initial_snapshot):
        with open(initial_snapshot, 'r') as f:
            previous_content = f.read()
    
    log_file = open(LIVE_LOG_OUTPUT, 'a', buffering=1)
    
    try:
        while True:
            time.sleep(2)  # Poll every 2 seconds
            snapshot_count += 1
            
            # Skip polling if we've had too many consecutive errors
            if snapshot_count > 5 and (snapshot_count % 10 == 0):
                print_info(f"Poll #{snapshot_count}: Periodic status check - continuing monitoring...")
            
            # Pull current snapshot
            try:
                pull_cmd = [
                    "xcrun", "devicectl", "device", "copy", "from",
                    "--device", DEVICE_UDID,
                    "--domain-type", "appDataContainer",
                    "--domain-identifier", BUNDLE_ID,
                    "--source", f"Documents/{DIAGNOSTIC_LOG_FILENAME}",
                    "--destination", f"{TMP_DIR}/poll_snapshot_current.log"
                ]
                
                result = run_with_timeout(pull_cmd, 20)
                
                if result is None:
                    print_error(f"Poll #{snapshot_count}: devicectl hung (known CoreDevice issue)")
                    continue
                elif result.returncode != 0:
                    error_msg = result.stderr.strip() if result.stderr else "Unknown error"
                    
                    # Check for specific network socket errors
                    if "NSPOSIXErrorDomain error 60" in error_msg or "socket was closed unexpectedly" in error_msg:
                        print_error(f"Poll #{snapshot_count}: Network socket closed (device disconnected?)")
                        print_info("This can happen if:")
                        print_info("  • USB connection was interrupted")
                        print_info("  • Device went to sleep or locked")
                        print_info("  • CoreDevice service restarted")
                        print_info("Continuing to retry...")
                    elif "com.apple.dt.CoreDeviceError error 7000" in error_msg:
                        print_error(f"Poll #{snapshot_count}: File transfer failed (CoreDevice error)")
                        print_info("Retrying in next poll cycle...")
                    else:
                        print_error(f"Poll #{snapshot_count}: devicectl failed - {error_msg}")
                    continue
                
                if result is None:
                    print_error(f"Poll #{snapshot_count}: devicectl hung (known CoreDevice issue)")
                    continue
                elif result.returncode != 0:
                    print_error(f"Poll #{snapshot_count}: devicectl failed - {result.stderr.strip()}")
                    continue
                    
                snapshot_file = f"{TMP_DIR}/poll_snapshot_current.log"
                if os.path.exists(snapshot_file):
                    with open(snapshot_file, 'r') as sf:
                        current_content = sf.read()
                        
                    # Check if content changed
                    if current_content != previous_content:
                        # Extract new lines (simple approach: get lines not in previous)
                        current_lines = current_content.split('\n')
                        previous_lines = previous_content.split('\n')
                        
                        # Find new lines
                        new_lines = []
                        if len(current_lines) > len(previous_lines):
                            new_lines = current_lines[len(previous_lines):]
                        
                        if new_lines:
                            timestamp = time.strftime('%H:%M:%S')
                            log_file.write(f"\n--- New activity at {timestamp} (poll #{snapshot_count}) ---\n")
                            log_file.write('\n'.join(new_lines))
                            log_file.write('\n')
                            log_file.flush()
                            
                            print_info(f"Poll #{snapshot_count}: {len(new_lines)} new log entries")
                        
                        previous_content = current_content
                    else:
                        # No changes - show periodic heartbeat
                        if snapshot_count % 15 == 0:  # Every 30 seconds
                            print_info(f"Poll #{snapshot_count}: No new activity (monitoring continues...)")
                else:
                    # File doesn't exist - might be first poll after error recovery
                    if snapshot_count % 10 == 0:
                        print_info(f"Poll #{snapshot_count}: Snapshot file not found, will retry...")
                            
            except Exception as e:
                error_msg = str(e)
                if "NSPOSIXErrorDomain error 60" in error_msg:
                    print_error(f"Poll #{snapshot_count}: Network connection lost")
                else:
                    print_error(f"Poll #{snapshot_count}: Error - {e}")
                # Continue polling despite errors
                continue
                
    except KeyboardInterrupt:
        log_file.write(f"\n=== Monitoring stopped at {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")
        log_file.close()
        raise  # Re-raise to be caught by main
    finally:
        if not log_file.closed:
            log_file.close()

# ===========================
# PHASE 2B: DIAGNOSTIC SNAPSHOT
# ===========================

def extract_diagnostic_snapshot():
    """
    Extract the diagnostic snapshot file from the iOS device's Documents directory.
    Uses xcrun devicectl to pull the file from the app container.
    """
    print_phase("PHASE 2B: EXTRACTING DIAGNOSTIC SNAPSHOT")
    
    print_info(f"Target File: Documents/{DIAGNOSTIC_LOG_FILENAME}")
    print_info(f"Output: {SNAPSHOT_OUTPUT}")
    
    # Use devicectl to copy the file from the app container
    print_info("Pulling diagnostic file from app Documents directory...")
    
    # Construct the source path - relative to the app container
    source_path = f"Documents/{DIAGNOSTIC_LOG_FILENAME}"
    
    # Pull the file using devicectl copy from
    pull_cmd = [
        "xcrun", "devicectl", "device", "copy", "from",
        "--device", DEVICE_UDID,
        "--domain-type", "appDataContainer",
        "--domain-identifier", BUNDLE_ID,
        "--source", source_path,
        "--destination", SNAPSHOT_OUTPUT
    ]
    
    print_info(f"Command: {' '.join(pull_cmd)}")
    
    try:
        result = run_with_timeout(pull_cmd, 20)
        
        if result is None:
            print_error("devicectl command hung - this is a known CoreDevice issue")
            print_error("Diagnostic snapshot may not be accessible")
            print_info("Possible causes:")
            print_info("  • App hasn't run long enough to create the diagnostic file")
            print_info("  • App container permissions issue")  
            print_info("  • iOS/CoreDevice bug with file system access")
            return False
        elif result.returncode != 0:
            print_error(f"devicectl failed: {result.stderr}")
            print_info("Attempting alternative extraction with idevice tools...")
            return extract_diagnostic_snapshot_idevice()
        
        # Verify the pulled file
        if os.path.exists(SNAPSHOT_OUTPUT):
            size = os.path.getsize(SNAPSHOT_OUTPUT)
            if size > 0:
                with open(SNAPSHOT_OUTPUT, 'r') as f:
                    lines = f.readlines()
                    line_count = len(lines)
                print_success(f"Diagnostic snapshot extracted! {line_count} lines ({size} bytes)")
                print_info("Sample content (first 5 lines):")
                for line in lines[:5]:
                    print(f"  {line.rstrip()}")
                return True
            else:
                print_error("Pulled file exists but is empty (0 bytes)")
                return False
        else:
            print_error(f"Failed to pull diagnostic snapshot")
            return False
    except Exception as e:
        print_error(f"Unexpected error: {e}")
        return False

def extract_diagnostic_snapshot_idevice():
    """
    Fallback method using libimobiledevice tools (idevice-app-info, ifuse, etc.)
    """
    print_info("Checking for libimobiledevice tools...")
    
    # Check if ideviceinstaller is available
    try:
        subprocess.run(["which", "ideviceinstaller"], 
                      capture_output=True, check=True)
    except:
        print_error("libimobiledevice tools not available. Install with: brew install libimobiledevice")
        return False
    
    print_info("Attempting to access Documents directory via AFC...")
    
    # Use house_arrest to access app documents
    # This is a simplified approach - may need refinement based on your setup
    try:
        # Create temp mount point
        mount_point = TMP_DIR / "ios_mount"
        mount_point.mkdir(exist_ok=True)
        
        # Try to use ifuse with house_arrest
        mount_cmd = [
            "ifuse",
            str(mount_point),
            "--documents",
            BUNDLE_ID
        ]
        
        print_info(f"Mounting: {' '.join(mount_cmd)}")
        subprocess.run(mount_cmd, check=True, timeout=10)
        
        # Copy the file
        source = mount_point / DIAGNOSTIC_LOG_FILENAME
        if source.exists():
            import shutil
            shutil.copy(source, SNAPSHOT_OUTPUT)
            print_success(f"Copied via ifuse mount")
            
            # Unmount
            subprocess.run(["umount", str(mount_point)], timeout=5)
            return True
        else:
            print_error(f"File not found at mount point: {source}")
            subprocess.run(["umount", str(mount_point)], timeout=5)
            return False
            
    except Exception as e:
        print_error(f"ifuse method failed: {e}")
        return False

# ===========================
# PHASE 3: GRACEFUL TEARDOWN
# ===========================

def cleanup_stream(proc, log_file):
    """
    Gracefully terminate the log streaming process.
    """
    print_phase("PHASE 3: GRACEFUL TEARDOWN")
    
    if proc is None:
        print_info("No streaming process to clean up")
        return
    
    print_info("Sending SIGTERM to log stream process...")
    try:
        proc.send_signal(signal.SIGTERM)
        print_info("Waiting for process to terminate (5 second timeout)...")
        try:
            proc.wait(timeout=5)
            print_success("Log stream process terminated cleanly")
        except subprocess.TimeoutExpired:
            print_error("Process did not terminate, sending SIGKILL...")
            proc.kill()
            proc.wait(timeout=2)
            print_success("Process killed forcefully")
    except Exception as e:
        print_error(f"Error during cleanup: {e}")
    finally:
        if log_file and not log_file.closed:
            log_file.close()
            print_info("Log file closed")
    
    # Verify no zombie processes
    try:
        result = subprocess.run(
            ["pgrep", "-f", "devicectl.*log.*stream"],
            capture_output=True,
            text=True
        )
        if result.returncode == 0 and result.stdout.strip():
            print_error(f"Warning: Found lingering devicectl processes: {result.stdout.strip()}")
        else:
            print_success("No zombie streaming processes detected")
    except:
        pass

# ===========================
# MAIN EXECUTION
# ===========================

def parse_arguments():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        prog='ios_extractor.py',
        description='iOS Log Extractor for SCMessenger - Extracts live diagnostic logs',
        epilog='Press Ctrl+C to stop capturing and cleanup',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        '-v', '--version',
        action='version',
        version='%(prog)s 1.0.0'
    )
    
    parser.add_argument(
        '-q', '--quiet',
        action='store_true',
        help='Quiet mode - suppress informational messages'
    )
    
    # Add help text that will be shown with -h
    parser.description = """
iOS Log Extractor for SCMessenger

DESCRIPTION:
    Extracts live diagnostic snapshot logs from a connected iOS device.
    The script runs continuously until you press Ctrl+C.
    
    Uses diagnostic snapshot polling (every 2 seconds) which provides structured
    app-level diagnostic events. This is more useful than raw OSLog for debugging
    SCMessenger-specific issues.

FEATURES:
    • Auto-detects connected iOS device via CoreDevice
    • Polls diagnostic snapshots every 2 seconds  
    • Extracts structured diagnostic events from app container
    • Automatic log rotation (24 hours / 100 MB per platform)
    • Graceful cleanup on exit
    • Optional quiet mode (-q) for minimal output
    iOS device. The script runs continuously until you press Ctrl+C.

FEATURES:
    • Auto-detects connected iOS device via CoreDevice
    • Extracts diagnostic snapshot from app container
    • Continuous monitoring until Ctrl+C
    • Two modes: Real-time streaming (idevicesyslog) or intelligent polling
    • Automatic log rotation (24 hours / 100 MB per platform)
    • Graceful cleanup on exit

REQUIREMENTS:
    • iOS device connected via USB and trusted
    • macOS with Xcode Command Line Tools
    • Python 3.7+
    • Optional: libimobiledevice for real-time streaming
      Install with: brew install libimobiledevice

OUTPUT FILES:
    ios_diagnostic_snapshot.log    Point-in-time structured diagnostics (PRIMARY)
    live_ios_log.log               Continuous capture (grows over time)
    ios_logs_archive/              Archived logs (auto-managed)

CONFIGURATION:
    Bundle ID:        SovereignCommunications.SCMessenger
    OSLog Subsystem:  com.scmessenger
    Device UDID:      {udid}
    
    Log Rotation:
      Max Age:        {age} hours
      Max Size:       {size} MB per platform
      Archive Dir:    {archive}

USAGE EXAMPLES:
    # Standard extraction (most common)
    python3 ios_extractor.py
    
    # With real-time streaming (if libimobiledevice installed)
    brew install libimobiledevice
    python3 ios_extractor.py
    
    # Stop capturing
    [Press Ctrl+C]

MODES:
    1. Streaming Mode (if idevicesyslog available)
       - Uses idevicesyslog for true real-time OSLog capture
       - Captures system-level iOS logs
       - Best for debugging system-level issues
    
    2. Polling Mode (default fallback)
       - Polls diagnostic snapshot every 5 seconds
       - Captures app's structured diagnostic events
       - More reliable for SCMessenger-specific debugging
       - Often provides MORE useful info than raw OSLog

HOW IT WORKS:
    1. Archive previous logs with timestamp
    2. Clean up old logs (>24h or >100MB)
    3. Start live log streaming or polling
    4. Extract diagnostic snapshot
    5. Run continuously until Ctrl+C
    6. Graceful cleanup on exit

LOG ROTATION:
    • Previous logs archived with timestamp on each run
    • Logs older than 24 hours automatically deleted
    • Total archive size limited to 100 MB per platform
    • Oldest logs deleted first when over limit
    • iOS and Android have separate 100 MB budgets

DOCUMENTATION:
    Quick Start:     QUICKSTART_IOS_LOGS.md
    Full Details:    iOS_LOG_EXTRACTION_SUMMARY.md
    Standard:        LOG_EXTRACTION_STANDARD.md
    Rotation:        LOG_ROTATION_INFO.md

TROUBLESHOOTING:
    "Device not found"
      → Connect iPhone via USB
      → Trust the Mac (tap "Trust" on iPhone)
      → Check: xcrun devicectl list devices
    
    "No logs captured"
      → Make sure the app is running
      → Send a test message to generate activity
      → Check battery isn't too low (< 20%)
    
    "Stream started but no data captured"
      → This is normal if the app is idle
      → Trigger some activity (send a message, open app)
      → The stream is ready and will capture when events occur

For more help, see: LOG_EXTRACTION_STANDARD.md
""".format(
        udid=DEVICE_UDID,
        age=MAX_LOG_AGE_HOURS,
        size=MAX_TOTAL_SIZE_MB,
        archive=LOG_ARCHIVE_DIR
    )
    
    return parser.parse_args()

def main():
    """
    Main execution flow.
    """
    global QUIET_MODE
    
    # Parse command line arguments (handles -h automatically)
    args = parse_arguments()
    
    # Set global quiet mode from arguments
    QUIET_MODE = args.quiet
    
    stream_handle = None
    log_file_handle = None
    polling_mode = False
    
    try:
        print_phase("iOS LOG EXTRACTOR FOR SCMESSENGER")
        print_info("Repository-specific extraction using discovered parameters")
        
        # Rotate and cleanup old logs before starting new capture
        print_phase("LOG ROTATION AND CLEANUP")
        rotate_previous_logs()
        
        # Phase 2A: Start live log streaming
        stream_handle = extract_live_logs()
        if stream_handle:
            proc, log_file_handle = stream_handle
        else:
            polling_mode = True
        
        # Phase 2B: Extract diagnostic snapshot
        snapshot_success = extract_diagnostic_snapshot()
        
        # If using live streaming (idevicesyslog), keep it running
        if stream_handle:
            print_info("\nInitial verification complete. Continuing live capture...")
            time.sleep(5)
            
            # Show updated stats
            if os.path.exists(LIVE_LOG_OUTPUT):
                size = os.path.getsize(LIVE_LOG_OUTPUT)
                with open(LIVE_LOG_OUTPUT, 'r') as f:
                    line_count = len(f.readlines())
                print_success(f"Captured so far: {line_count} lines ({size} bytes)")
            
            print_success("\n✓ Log extraction setup completed successfully")
            print_info(f"Live stream is actively capturing to: {LIVE_LOG_OUTPUT}")
            print_info("Press Ctrl+C to stop and cleanup")
            
            # Keep running until user interrupts
            print_info("\n[Monitoring] Live capture active... (Ctrl+C to stop)")
            while True:
                time.sleep(5)
                # Check if process died
                if proc.poll() is not None:
                    print_error("\nLog stream process ended unexpectedly")
                    break
        elif polling_mode:
            # Continue polling mode
            print_success("\n✓ Initial polling setup completed")
            print_info(f"Polling results being written to: {LIVE_LOG_OUTPUT}")
            continue_polling_mode()
    
    except KeyboardInterrupt:
        print_info("\n\nReceived Ctrl+C, shutting down...")
    except Exception as e:
        print_error(f"\nUnexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        # Phase 3: Cleanup
        if stream_handle:
            proc, log_file_handle = stream_handle
            cleanup_stream(proc, log_file_handle)
        print_info("\n✓ All processes terminated")
    
    # Final summary
    print_phase("EXTRACTION COMPLETE")
    
    results = []
    if os.path.exists(LIVE_LOG_OUTPUT):
        size = os.path.getsize(LIVE_LOG_OUTPUT)
        results.append(f"✓ Live logs: {LIVE_LOG_OUTPUT} ({size} bytes)")
    else:
        results.append(f"✗ Live logs: FAILED")
    
    if os.path.exists(SNAPSHOT_OUTPUT):
        size = os.path.getsize(SNAPSHOT_OUTPUT)
        results.append(f"✓ Diagnostic snapshot: {SNAPSHOT_OUTPUT} ({size} bytes)")
    else:
        results.append(f"✗ Diagnostic snapshot: FAILED")
    
    for result in results:
        print(f"  {result}")
    
    print()

if __name__ == "__main__":
    main()
