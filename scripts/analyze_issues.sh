#!/bin/bash
echo "Issue Analysis from Logs"
echo "========================"
echo ""

LOG="scripts/android_current.log"

echo "1. Error/Exception Summary:"
grep -E "E/|Exception|Error|FATAL" "$LOG" | cut -d: -f4- | sort | uniq -c | sort -rn | head -10
echo ""

echo "2. Message Send Activity:"
grep "SEND_MSG" "$LOG" | wc -l | xargs echo "  Send attempts:"
grep "message_prepared_local_history_written" "$LOG" | wc -l | xargs echo "  Messages saved to history:"
grep "delivery_state.*delivered" "$LOG" | wc -l | xargs echo "  Delivery confirmations:"
echo ""

echo "3. ChatViewModel Activity:"
grep -c "ChatViewModel" "$LOG" | xargs echo "  ChatViewModel log entries:"
grep -c "ConversationsViewModel" "$LOG" | xargs echo "  ConversationsViewModel log entries:"
echo ""

echo "4. ID Resolution Issues:"
grep -E "normalized|resolved|canonical" "$LOG" | head -5
echo ""

echo "5. Recent Errors (last 20):"
grep -E "E/|Exception|Error" "$LOG" | tail -20
echo ""

echo "6. Performance Warnings:"
grep -iE "slow|timeout|hang|freeze|anr" "$LOG" | head -10
