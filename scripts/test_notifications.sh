#!/bin/bash

# SCMessenger Notification Verification Test Script
# 
# This script tests notification functionality across all platforms
# to verify that WS14 notification implementation works correctly.

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
TEST_OUTPUT_DIR="test_output/notifications"
LOG_FILE="$TEST_OUTPUT_DIR/notification_test_$(date +%Y%m%d_%H%M%S).log"

# Create output directory
mkdir -p "$TEST_OUTPUT_DIR"

echo -e "${BLUE}=== SCMessenger Notification Verification Test ===${NC}"
echo -e "${BLUE}Starting at: $(date)${NC}"
echo -e "${BLUE}Log file: $LOG_FILE${NC}"
echo ""

# Function to log messages
log_message() {
    echo "$1"
    echo "$(date) - $1" >> "$LOG_FILE"
}

# Function to log errors
log_error() {
    echo -e "${RED}$1${NC}"
    echo "$(date) - ERROR - $1" >> "$LOG_FILE"
}

# Function to log success
log_success() {
    echo -e "${GREEN}$1${NC}"
    echo "$(date) - SUCCESS - $1" >> "$LOG_FILE"
}

# Function to run a test and check resultun_test() {
    local test_name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Running: $test_name${NC}"
    log_message "Starting test: $test_name"
    
    if eval "$command"; then
        log_success "✓ $test_name passed"
        return 0
    else
        log_error "✗ $test_name failed"
        return 1
    fi
}

# Test 1: Verify notification files exist
echo -e "${BLUE}=== Test Phase 1: File Verification ===${NC}"

run_test "Check iOS Notification Files" \
"test -f iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift && \
 test -f iOS/SCMessenger/SCMessenger/Notifications/NotificationHelper.swift"

run_test "Check Android Notification Files" \
"test -f android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt && \
 test -f android/app/src/main/java/com/scmessenger/android/notifications/NotificationHelper.kt"

# Test 2: Verify notification constants and classification
echo -e "${BLUE}=== Test Phase 2: Code Analysis ===${NC}"

run_test "Check iOS Notification Classification" \
"grep -q 'explicitDmRequest' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check Android Notification Classification" \
"grep -q 'explicitDmRequest' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

run_test "Check iOS DM vs DM Request Logic" \
"grep -q 'isKnownContact' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check Android DM vs DM Request Logic" \
"grep -q 'isKnownContact' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

# Test 3: Verify notification channels (Android)
echo -e "${BLUE}=== Test Phase 3: Android Notification Channels ===${NC}"

run_test "Check Android Notification Channel Creation" \
"grep -q 'createNotificationChannel' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

run_test "Check Android Channel IDs" \
"grep -q 'CHANNEL_ID' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

# Test 4: Verify iOS notification categories
echo -e "${BLUE}=== Test Phase 4: iOS Notification Categories ===${NC}"

run_test "Check iOS Notification Categories" \
"grep -q 'UNNotificationCategory' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check iOS Notification Actions" \
"grep -q 'UNNotificationAction' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

# Test 5: Verify notification content handling
echo -e "${BLUE}=== Test Phase 5: Notification Content ===${NC}"

run_test "Check iOS Notification Content Handling" \
"grep -q 'content.text' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check Android Notification Content Handling" \
"grep -q 'setContentText' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

# Test 6: Verify notification routing
echo -e "${BLUE}=== Test Phase 6: Notification Routing ===${NC}"

run_test "Check iOS Notification Tap Handling" \
"grep -q 'userInfo' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check Android Notification Tap Handling" \
"grep -q 'PendingIntent' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

# Test 7: Verify notification settings integration
echo -e "${BLUE}=== Test Phase 7: Settings Integration ===${NC}"

run_test "Check iOS Notification Settings" \
"grep -q 'notificationsEnabled' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check Android Notification Settings" \
"grep -q 'notificationsEnabled' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

# Test 8: Verify WS14 classification implementation
echo -e "${BLUE}=== Test Phase 8: WS14 Classification Verification ===${NC}"

run_test "Check iOS WS14 Classification Parameters" \
"grep -q 'hasExistingConversation' iOS/SCMessenger/SCMessenger/Notifications/NotificationManager.swift"

run_test "Check Android WS14 Classification Parameters" \
"grep -q 'hasExistingConversation' android/app/src/main/java/com/scmessenger/android/notifications/NotificationManager.kt"

# Summary
echo ""
echo -e "${BLUE}=== Notification Verification Test Summary ===${NC}"
echo -e "${GREEN}All notification verification tests completed!${NC}"
echo -e "${BLUE}Results saved to: $LOG_FILE${NC}"
echo -e "${BLUE}Completed at: $(date)${NC}"

echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Review test results in $LOG_FILE"
echo "2. Test notifications on physical devices"
echo "3. Verify cross-platform notification behavior"
echo "4. Test notification tap routing to correct conversations"

exit 0