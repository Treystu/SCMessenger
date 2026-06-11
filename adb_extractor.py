#!/usr/bin/env python3
"""
Android Log Extractor for SCMessenger
Extracts live streaming Logcat and diagnostic snapshot logs from connected device.

CRITICAL: The app is already running and logs are actively streaming.
"""

import subprocess
import time
import signal
import sys
import os
import argparse
from datetime import datetime

# Phase 1: Discovered Parameters from Codebase Analysis
PACKAGE_NAME = "com.scmessenger.android"
PRIMARY_TAGS = ["SmartTransportRouter", "StorageManager", "FileLoggingTree"]
DIAGNOSTIC_LOG_PATH = "/data/data/com.scmessenger.android/files/mesh_diagnostics.log"
DIAGNOSTIC_ROTATED_LOGS = [
    "/data/data/com.scmessenger.android/files/mesh_diagnostics.log.1",
    "/data/data/com.scmessenger.android/files/mesh_diagnostics.log.2",
    "/data/data/com.scmessenger.android/files/mesh_diagnostics.log.3",
    "/data/data/com.scmessenger.android/files/mesh_diagnostics.log.4",
    "/data/data/com.scmessenger.android/files/mesh_diagnostics.log.5",
]

# Output files
LIVE_LOGCAT_FILE = "live_logcat.log"
DIAGNOSTIC_SNAPSHOT_DIR = "diagnostic_snapshots"

# Log rotation settings
LOG_ARCHIVE_DIR = "android_logs_archive"
MAX_LOG_AGE_HOURS = 24
MAX_TOTAL_SIZE_MB = 100

# Global quiet mode flag (set by command line argument)
QUIET_MODE = False

def print_info(msg):
    """Print informational message unless in quiet mode"""
    if not QUIET_MODE:
        print(msg)

def print_error(msg):
    """Always print errors, even in quiet mode"""
    print(msg)

def print_success(msg):
    """Print success message unless in quiet mode"""
    if not QUIET_MODE:
        print(msg)

class LogExtractor:
    def __init__(self):
        self.logcat_process = None
        self.start_time = datetime.now()
        self.log_archive_path = os.path.join(os.getcwd(), LOG_ARCHIVE_DIR)
        
        # Create archive directory
        os.makedirs(self.log_archive_path, exist_ok=True)
        
    def cleanup(self):
        """Gracefully terminate background processes"""
        if self.logcat_process and self.logcat_process.poll() is None:
            print("\n[Cleanup] Terminating logcat subprocess...")
            self.logcat_process.send_signal(signal.SIGTERM)
            try:
                self.logcat_process.wait(timeout=5)
                print("[Cleanup] ✓ Logcat process terminated cleanly")
            except subprocess.TimeoutExpired:
                print("[Cleanup] ⚠ SIGTERM timeout, sending SIGKILL...")
                self.logcat_process.kill()
                self.logcat_process.wait()
                print("[Cleanup] ✓ Logcat process killed")
    
    def rotate_previous_logs(self):
        """
        Archive previous log files with timestamp before starting new capture.
        Implements log rotation: keeps logs for 24 hours or 100MB total (whichever comes first).
        """
        import shutil
        
        timestamp = self.start_time.strftime("%Y%m%d_%H%M%S")
        
        # Archive existing live logcat if it exists and has content
        if os.path.exists(LIVE_LOGCAT_FILE) and os.path.getsize(LIVE_LOGCAT_FILE) > 0:
            archived_name = f"live_logcat_{timestamp}.log"
            archived_path = os.path.join(self.log_archive_path, archived_name)
            shutil.move(LIVE_LOGCAT_FILE, archived_path)
            print(f"[Rotation] Archived previous logcat as: {archived_name}")
        
        # Archive existing diagnostic snapshots directory
        if os.path.exists(DIAGNOSTIC_SNAPSHOT_DIR) and os.listdir(DIAGNOSTIC_SNAPSHOT_DIR):
            archived_name = f"diagnostic_snapshots_{timestamp}"
            archived_path = os.path.join(self.log_archive_path, archived_name)
            shutil.move(DIAGNOSTIC_SNAPSHOT_DIR, archived_path)
            print(f"[Rotation] Archived previous diagnostic snapshots as: {archived_name}/")
        
        # Recreate diagnostic snapshot dir
        os.makedirs(DIAGNOSTIC_SNAPSHOT_DIR, exist_ok=True)
        
        # Clean up old logs
        self.cleanup_old_logs()
    
    def cleanup_old_logs(self):
        """
        Remove logs older than 24 hours OR enforce 100MB total size limit.
        Deletes oldest logs first until both constraints are met.
        """
        # Get all log files and directories in archive
        archive_items = []
        for item_name in os.listdir(self.log_archive_path):
            item_path = os.path.join(self.log_archive_path, item_name)
            
            # Calculate total size (file or directory)
            if os.path.isfile(item_path):
                size = os.path.getsize(item_path)
            else:
                size = sum(
                    os.path.getsize(os.path.join(dirpath, filename))
                    for dirpath, dirnames, filenames in os.walk(item_path)
                    for filename in filenames
                )
            
            stat = os.stat(item_path)
            age_hours = (time.time() - stat.st_mtime) / 3600
            
            archive_items.append({
                'path': item_path,
                'name': item_name,
                'size': size,
                'mtime': stat.st_mtime,
                'age_hours': age_hours,
                'is_dir': os.path.isdir(item_path)
            })
        
        if not archive_items:
            return
        
        # Sort by age (oldest first)
        archive_items.sort(key=lambda x: x['mtime'])
        
        # Step 1: Remove items older than 24 hours
        items_to_delete = []
        for item in archive_items:
            if item['age_hours'] > MAX_LOG_AGE_HOURS:
                items_to_delete.append(item)
        
        if items_to_delete:
            print(f"[Cleanup] Removing {len(items_to_delete)} item(s) older than {MAX_LOG_AGE_HOURS} hours...")
            for item in items_to_delete:
                if item['is_dir']:
                    import shutil
                    shutil.rmtree(item['path'])
                else:
                    os.unlink(item['path'])
                archive_items.remove(item)
        
        # Step 2: Enforce 100MB total size limit
        max_size_bytes = MAX_TOTAL_SIZE_MB * 1024 * 1024
        total_size = sum(item['size'] for item in archive_items)
        
        if total_size > max_size_bytes:
            print(f"[Cleanup] Total log size ({total_size / 1024 / 1024:.1f} MB) exceeds {MAX_TOTAL_SIZE_MB} MB limit...")
            
            # Delete oldest items until under limit
            while archive_items and total_size > max_size_bytes:
                oldest = archive_items.pop(0)
                if oldest['is_dir']:
                    import shutil
                    shutil.rmtree(oldest['path'])
                else:
                    os.unlink(oldest['path'])
                total_size -= oldest['size']
                print(f"[Cleanup] Deleted old item: {oldest['name']} ({oldest['size'] / 1024:.1f} KB)")
            
            print(f"[Cleanup] ✓ Log cleanup complete. Total size now: {total_size / 1024 / 1024:.1f} MB")
                
    def verify_device_connected(self):
        """Verify ADB device is connected and authorized"""
        print("[Verification] Checking ADB device connection...")
        result = subprocess.run(
            ["adb", "devices"],
            capture_output=True,
            text=True
        )
        
        devices = [line for line in result.stdout.strip().split('\n')[1:] if line.strip()]
        if not devices or "unauthorized" in result.stdout:
            print("[ERROR] ✗ No authorized ADB device found")
            return False
            
        print(f"[Verification] ✓ Device connected: {devices[0]}")
        return True
        
    def verify_app_running(self):
        """Verify target app is currently running"""
        print(f"[Verification] Checking if {PACKAGE_NAME} is running...")
        result = subprocess.run(
            ["adb", "shell", f"pidof {PACKAGE_NAME}"],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0 or not result.stdout.strip():
            print(f"[WARNING] ⚠ App may not be running (no PID found)")
            return False
            
        pid = result.stdout.strip()
        print(f"[Verification] ✓ App is running (PID: {pid})")
        return True
    
    def start_diagnostic_polling(self):
        """
        Start diagnostic snapshot polling (every 2 seconds).
        This is the DEFAULT and RECOMMENDED method as it provides structured
        app-level diagnostic events similar to iOS.
        """
        print("\n" + "="*70)
        print("PHASE 2.1: DIAGNOSTIC SNAPSHOT POLLING")
        print("="*70)
        
        print_info("[Polling] Initializing diagnostic snapshot polling (every 2 seconds)...")
        print_info("[Polling] This method provides app-level structured diagnostics")
        
        # Open output file
        log_file = open(LIVE_LOGCAT_FILE, 'w', buffering=1)
        log_file.write("=== Android Diagnostic Log Monitoring (Polling Mode) ===\n")
        log_file.write(f"Started at: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        log_file.write(f"Package: {PACKAGE_NAME}\n")
        log_file.write("Polling every 2 seconds. Press Ctrl+C to stop.\n")
        log_file.write("Note: This captures app-generated diagnostic events.\n\n")
        log_file.flush()
        
        previous_content = ""
        snapshot_count = 0
        
        print_info("[Polling] Pulling initial diagnostic snapshot...")
        
        # Initial snapshot
        try:
            result = subprocess.run(
                ["adb", "shell", f"run-as {PACKAGE_NAME} cat files/{os.path.basename(DIAGNOSTIC_LOG_PATH)}"],
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode == 0 and result.stdout:
                previous_content = result.stdout
                log_file.write(f"--- Initial Snapshot at {self.start_time.strftime('%H:%M:%S')} ---\n")
                log_file.write(previous_content)
                log_file.write('\n')
                log_file.flush()
                
                line_count = len(previous_content.split('\n'))
                print_success(f"[Polling] ✓ Initial snapshot captured: {line_count} lines")
                print_info("[Polling] These are app-generated diagnostic events (delivery attempts, BLE rx/tx, etc.)")
            else:
                print_info("[Polling] ⚠ No diagnostic log found yet (app may need to generate events)")
                
        except Exception as e:
            print_info(f"[Polling] ⚠ Initial snapshot failed: {e}")
        
        print_success(f"[Polling] ✓ Polling mode initialized: {LIVE_LOGCAT_FILE}")
        print_info("[Polling] Diagnostic logs contain: delivery_attempt, ble_rx/tx, peer_identified, relay_state, etc.")
        
        return log_file, previous_content, snapshot_count
        
    def start_live_logcat_stream(self):
        """
        Phase 2.1: Start targeted streaming Logcat
        Filter by package name and specific transport/mesh tags
        """
        print("\n" + "="*70)
        print("PHASE 2.1: LIVE LOGCAT STREAMING")
        print("="*70)
        
        # Get the PID first
        pid_result = subprocess.run(
            ["adb", "shell", f"pidof {PACKAGE_NAME}"],
            capture_output=True,
            text=True
        )
        
        if pid_result.returncode != 0 or not pid_result.stdout.strip():
            print("[WARNING] Could not get app PID, streaming all logs with package filter")
            cmd = [
                "adb", "logcat",
                "-v", "threadtime",
                f"{PACKAGE_NAME}:*",
                "*:S"
            ]
        else:
            pid = pid_result.stdout.strip()
            print(f"[Logcat] Filtering by PID: {pid}")
            cmd = [
                "adb", "logcat",
                "-v", "threadtime",
                "--pid", pid
            ]
        
        print(f"[Logcat] Command: {' '.join(cmd)}")
        print(f"[Logcat] Output file: {LIVE_LOGCAT_FILE}")
        
        # Clear old logcat buffer
        subprocess.run(["adb", "logcat", "-c"], capture_output=True)
        
        # Open output file
        logcat_file = open(LIVE_LOGCAT_FILE, 'w', buffering=1)  # Line buffered
        
        # Start subprocess
        self.logcat_process = subprocess.Popen(
            cmd,
            stdout=logcat_file,
            stderr=subprocess.PIPE,
            text=True,
            preexec_fn=os.setsid if sys.platform != 'win32' else None
        )
        
        return logcat_file
        
    def trigger_app_activity(self):
        """Trigger app activity to generate logs for verification"""
        print("\n[Trigger] Attempting to trigger app activity for log verification...")
        
        # Method 1: Bring app to foreground
        result = subprocess.run(
            ["adb", "shell", f"am start -n {PACKAGE_NAME}/.ui.MainActivity"],
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            print("[Trigger] ✓ Brought app to foreground")
        
        time.sleep(0.5)
        
        # Method 2: Send a broadcast (if app has receivers)
        subprocess.run(
            ["adb", "shell", f"am broadcast -a android.intent.action.BOOT_COMPLETED -p {PACKAGE_NAME}"],
            capture_output=True
        )
        
        time.sleep(0.5)
    
    def verify_live_stream(self, logcat_file):
        """
        Phase 2.1b: Live Verification
        Read the stream for 5 seconds and confirm app logs are captured
        """
        print("\n[Verification] Live stream verification (8 seconds with activity trigger)...")
        
        # Wait a moment for initial stream
        time.sleep(1)
        
        # Trigger app activity
        self.trigger_app_activity()
        
        # Monitor the stream
        verification_start = time.time()
        lines_captured = 0
        app_lines_captured = 0
        last_size = 0
        
        # Monitor the file for 5 more seconds
        while time.time() - verification_start < 5.0:
            time.sleep(0.5)
            
            # Check if process died
            if self.logcat_process.poll() is not None:
                print(f"[ERROR] ✗ Logcat process died (exit code: {self.logcat_process.returncode})")
                stderr = self.logcat_process.stderr.read()
                if stderr:
                    print(f"[ERROR] stderr: {stderr}")
                return False
                
            # Check file size growth
            try:
                size = os.path.getsize(LIVE_LOGCAT_FILE)
                if size > last_size:
                    print(f"[Verification] Stream active: {size} bytes (+{size - last_size})")
                    last_size = size
                    
                if size > 0:
                    # Read and count lines
                    with open(LIVE_LOGCAT_FILE, 'r') as f:
                        lines = f.readlines()
                        lines_captured = len(lines)
                        # Count lines that contain package name or key tags
                        app_lines_captured = sum(1 for line in lines 
                                                if PACKAGE_NAME in line or 
                                                any(tag in line for tag in PRIMARY_TAGS))
            except Exception as e:
                print(f"[WARNING] Error reading log file: {e}")
                
        # Final assessment
        print(f"[Verification] Total lines captured: {lines_captured}")
        print(f"[Verification] App-specific lines: {app_lines_captured}")
        
        if app_lines_captured > 0:
            print(f"[Verification] ✓ SUCCESS: Live stream is capturing app logs")
            return True
        elif lines_captured > 0:
            print(f"[Verification] ⚠ WARNING: Capturing logs but no app-specific content yet")
            print(f"[Verification]   This may be normal if app is idle")
            return True
        else:
            print(f"[Verification] ⚠ WARNING: App appears idle, no new logs generated")
            print(f"[Verification]   Stream is properly configured and will capture when activity occurs")
            return True  # Don't fail - stream is ready
            
    def pull_diagnostic_snapshots(self):
        """
        Phase 2.2: Pull local diagnostic log snapshots
        Extract mesh_diagnostics.log and rotated backups
        """
        print("\n" + "="*70)
        print("PHASE 2.2: DIAGNOSTIC SNAPSHOT EXTRACTION")
        print("="*70)
        
        # Create output directory
        os.makedirs(DIAGNOSTIC_SNAPSHOT_DIR, exist_ok=True)
        timestamp = self.start_time.strftime("%Y%m%d_%H%M%S")
        
        pulled_files = []
        
        # Pull primary diagnostic log
        logs_to_pull = [DIAGNOSTIC_LOG_PATH] + DIAGNOSTIC_ROTATED_LOGS
        
        for log_path in logs_to_pull:
            filename = os.path.basename(log_path)
            local_path = os.path.join(DIAGNOSTIC_SNAPSHOT_DIR, f"{timestamp}_{filename}")
            
            print(f"\n[Pull] Extracting: {log_path}")
            
            # Use run-as for non-root access to app private directory
            result = subprocess.run(
                ["adb", "shell", f"run-as {PACKAGE_NAME} cat {log_path.replace('/data/data/com.scmessenger.android/', '')}"],
                capture_output=True
            )
            
            if result.returncode == 0 and len(result.stdout) > 0:
                with open(local_path, 'wb') as f:
                    f.write(result.stdout)
                    
                size = os.path.getsize(local_path)
                print(f"[Pull] ✓ SUCCESS: {filename} ({size} bytes)")
                pulled_files.append((filename, local_path, size))
            else:
                print(f"[Pull] ⚠ SKIP: {filename} (not found or empty)")
                
        return pulled_files
        
    def verify_snapshots(self, pulled_files):
        """Phase 2.2b: Verify pulled diagnostic files"""
        print("\n[Verification] Diagnostic snapshot verification...")
        
        if not pulled_files:
            print("[Verification] ✗ FAILURE: No diagnostic files pulled")
            return False
            
        total_size = sum(size for _, _, size in pulled_files)
        print(f"[Verification] Files pulled: {len(pulled_files)}")
        print(f"[Verification] Total size: {total_size} bytes")
        
        for filename, path, size in pulled_files:
            print(f"[Verification]   - {filename}: {size} bytes")
            
        if total_size > 0:
            print("[Verification] ✓ SUCCESS: Diagnostic snapshots extracted")
            return True
        else:
            print("[Verification] ✗ FAILURE: All files empty")
            return False
    
    def continue_diagnostic_polling(self, log_file, previous_content, snapshot_count):
        """
        Continue polling diagnostic snapshots until Ctrl+C.
        Polls every 2 seconds for near-real-time monitoring.
        """
        print_info("\n[Monitoring] Polling mode active - checking every 2 seconds... (Ctrl+C to stop)")
        
        try:
            while True:
                time.sleep(2)  # Poll every 2 seconds
                snapshot_count += 1
                
                # Pull current snapshot
                try:
                    result = subprocess.run(
                        ["adb", "shell", f"run-as {PACKAGE_NAME} cat files/{os.path.basename(DIAGNOSTIC_LOG_PATH)}"],
                        capture_output=True,
                        text=True,
                        timeout=5
                    )
                    
                    if result.returncode == 0 and result.stdout:
                        current_content = result.stdout
                        
                        # Check if content changed
                        if current_content != previous_content:
                            # Extract new lines
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
                                
                                print_info(f"[Polling] Poll #{snapshot_count}: {len(new_lines)} new log entries")
                            
                            previous_content = current_content
                        else:
                            # No changes - show periodic heartbeat
                            if snapshot_count % 15 == 0:  # Every 30 seconds
                                print_info(f"[Polling] Poll #{snapshot_count}: No new activity (monitoring continues...)")
                                
                except subprocess.TimeoutExpired:
                    print_error(f"[Polling] ✗ Poll #{snapshot_count}: Timeout pulling snapshot")
                except Exception as e:
                    print_error(f"[Polling] ✗ Poll #{snapshot_count}: Error - {e}")
                    
        except KeyboardInterrupt:
            log_file.write(f"\n=== Monitoring stopped at {time.strftime('%Y-%m-%d %H:%M:%S')} ===\n")
            log_file.close()
            raise  # Re-raise to be caught by main
        finally:
            if not log_file.closed:
                log_file.close()
    
    def generate_summary(self, pulled_files):
        """Generate extraction summary"""
        duration = (datetime.now() - self.start_time).total_seconds()
        
        print("\n" + "="*70)
        print("EXTRACTION SUMMARY")
        print("="*70)
        print(f"Duration: {duration:.1f} seconds")
        print(f"Timestamp: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"\nLive Logcat Stream:")
        print(f"  File: {LIVE_LOGCAT_FILE}")
        if os.path.exists(LIVE_LOGCAT_FILE):
            size = os.path.getsize(LIVE_LOGCAT_FILE)
            print(f"  Size: {size} bytes")
            print(f"  Status: {'✓ Active (still streaming)' if self.logcat_process and self.logcat_process.poll() is None else '✓ Completed'}")
        
        print(f"\nDiagnostic Snapshots:")
        print(f"  Directory: {DIAGNOSTIC_SNAPSHOT_DIR}/")
        print(f"  Files pulled: {len(pulled_files)}")
        for filename, path, size in pulled_files:
            print(f"    - {filename}: {size} bytes")
            
        print("\n" + "="*70)

def parse_arguments():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        prog='adb_extractor.py',
        description='Android Log Extractor for SCMessenger - Extracts live diagnostic logs',
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
    
    parser.add_argument(
        '--logcat',
        action='store_true',
        help='Use logcat streaming instead of diagnostic polling (legacy mode)'
    )
    
    # Add comprehensive help text
    parser.description = """
Android Log Extractor for SCMessenger

DESCRIPTION:
    Extracts live diagnostic snapshot logs from a connected Android device.
    The script runs continuously until you press Ctrl+C.
    
    By default, uses diagnostic snapshot polling (every 2 seconds) which provides
    structured app-level diagnostic events. This is more useful than raw logcat
    for debugging SCMessenger-specific issues.

FEATURES:
    • Auto-detects connected Android device via ADB
    • Polls diagnostic snapshots every 2 seconds (default mode)
    • Extracts structured diagnostic events from app storage
    • Automatic log rotation (24 hours / 100 MB per platform)
    • Graceful cleanup on exit
    • Optional quiet mode (-q) for minimal output

REQUIREMENTS:
    • Android device connected via USB with USB debugging enabled
    • ADB installed and in PATH
    • Python 3.7+

OUTPUT FILES:
    live_logcat.log               Filtered logcat stream (PRIMARY)
    diagnostic_snapshots/         Structured diagnostic files
    android_logs_archive/         Archived logs (auto-managed)

CONFIGURATION:
    Package Name:     {package}
    Primary Tags:     {tags}
    
    Log Rotation:
      Max Age:        {age} hours
      Max Size:       {size} MB per platform
      Archive Dir:    {archive}

USAGE EXAMPLES:
    # Standard extraction (most common)
    python3 adb_extractor.py
    
    # Stop capturing
    [Press Ctrl+C]

HOW IT WORKS:
    1. Archive previous logs with timestamp
    2. Clean up old logs (>24h or >100MB)
    3. Verify ADB device connection
    4. Start PID-filtered logcat stream
    5. Extract diagnostic snapshots from app storage
    6. Run continuously until Ctrl+C
    7. Graceful cleanup on exit

LOG ROTATION:
    • Previous logs archived with timestamp on each run
    • Logs older than 24 hours automatically deleted
    • Total archive size limited to 100 MB per platform
    • Oldest logs deleted first when over limit
    • iOS and Android have separate 100 MB budgets

LOGCAT FILTERING:
    • PID-based filtering for app-specific logs
    • Captures:
      - App lifecycle events
      - Transport router decisions
      - Storage operations
      - Core messaging flow
      - SmartTransportRouter, StorageManager, FileLoggingTree tags

DIAGNOSTIC SNAPSHOTS:
    • Extracted from app private storage using run-as
    • Includes:
      - mesh_diagnostics.log (primary)
      - Rotated backups (.log.1, .log.2, etc.)
    • Stored in diagnostic_snapshots/ directory

DOCUMENTATION:
    Standard:        LOG_EXTRACTION_STANDARD.md
    Rotation:        LOG_ROTATION_INFO.md
    Quick Ref:       LOG_EXTRACTION_QUICK_REF.md

TROUBLESHOOTING:
    "No authorized ADB device"
      → Connect Android device via USB
      → Enable USB debugging in Developer Options
      → Check device screen for authorization prompt
      → Verify: adb devices
    
    "App may not be running"
      → Launch the SCMessenger app manually
      → Script will continue monitoring when app starts
    
    "Permission denied" extracting diagnostics
      → Ensure app is debuggable
      → Check USB debugging is enabled
      → Try: adb shell run-as {package} ls

VERIFICATION:
    The script automatically verifies:
    • Device connection and authorization
    • App is running (warns if not, but continues)
    • Live stream is capturing data
    • Diagnostic files are extracted successfully

For more help, see: LOG_EXTRACTION_STANDARD.md
""".format(
        package=PACKAGE_NAME,
        tags=', '.join(PRIMARY_TAGS),
        age=MAX_LOG_AGE_HOURS,
        size=MAX_TOTAL_SIZE_MB,
        archive=LOG_ARCHIVE_DIR
    )
    
    return parser.parse_args()
        
def main():
    global QUIET_MODE
    
    # Parse command line arguments (handles -h automatically)
    args = parse_arguments()
    
    # Set global quiet mode from arguments
    QUIET_MODE = args.quiet
    
    extractor = LogExtractor()
    logcat_file = None
    polling_data = None
    
    try:
        if not QUIET_MODE:
            print("="*70)
            print("SCMESSENGER ANDROID LOG EXTRACTOR")
            print("="*70)
            print(f"Package: {PACKAGE_NAME}")
            if args.logcat:
                print("Mode: Legacy logcat streaming")
            else:
                print("Mode: Diagnostic snapshot polling (default)")
            print(f"Started: {extractor.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
            print("="*70)
        
        # Rotate and cleanup old logs before starting new capture
        if not QUIET_MODE:
            print("\n" + "="*70)
            print("LOG ROTATION AND CLEANUP")
            print("="*70)
        extractor.rotate_previous_logs()
        
        # Phase 1: Verification
        if not extractor.verify_device_connected():
            sys.exit(1)
            
        extractor.verify_app_running()  # Warning only, not fatal
        
        # Phase 2: Choose extraction mode
        if args.logcat:
            # Legacy logcat streaming mode
            logcat_file = extractor.start_live_logcat_stream()
            time.sleep(1)  # Brief delay for subprocess to start
            
            if not extractor.verify_live_stream(logcat_file):
                if not QUIET_MODE:
                    print("\n[WARNING] Stream verification uncertain, but continuing...")
                    
            # Phase 2.2: Diagnostic Snapshots
            pulled_files = extractor.pull_diagnostic_snapshots()
            
            # Continuous monitoring
            if not QUIET_MODE:
                print("\n" + "="*70)
                print("CONTINUOUS MONITORING (LOGCAT MODE)")
                print("="*70)
                print("[Monitoring] Live logcat active... (Ctrl+C to stop)")
            
            # Keep running until interrupted
            while True:
                time.sleep(1)
        else:
            # Default: Diagnostic polling mode (like iOS)
            log_file, previous_content, snapshot_count = extractor.start_diagnostic_polling()
            
            # Pull initial diagnostic snapshots for completeness
            pulled_files = extractor.pull_diagnostic_snapshots()
            
            if not QUIET_MODE:
                print("\n✓ Initial polling setup completed")
                print(f"✓ Polling results being written to: {LIVE_LOGCAT_FILE}")
            
            # Start continuous polling
            extractor.continue_diagnostic_polling(log_file, previous_content, snapshot_count)
            
    except KeyboardInterrupt:
        if not QUIET_MODE:
            print("\nReceived Ctrl+C, shutting down...")
        
        # Cleanup
        extractor.cleanup()
        
        if not QUIET_MODE:
            print("\n✓ All processes terminated")
            
            # Show final summary
            print("\n" + "="*70)
            print("EXTRACTION COMPLETE")
            print("="*70)
            
            # Show file sizes
            if os.path.exists(LIVE_LOGCAT_FILE):
                size = os.path.getsize(LIVE_LOGCAT_FILE)
                print(f"  ✓ Live logs: {LIVE_LOGCAT_FILE} ({size} bytes)")
            
            if os.path.exists(DIAGNOSTIC_SNAPSHOT_DIR):
                import glob
                files = glob.glob(f"{DIAGNOSTIC_SNAPSHOT_DIR}/*")
                if files:
                    total_size = sum(os.path.getsize(f) for f in files)
                    print(f"  ✓ Diagnostic snapshots: {DIAGNOSTIC_SNAPSHOT_DIR}/ ({len(files)} files, {total_size} bytes)")
        
    except Exception as e:
        print(f"\n[ERROR] Unexpected error: {e}")
        extractor.cleanup()
        sys.exit(1)
        
if __name__ == "__main__":
    main()
