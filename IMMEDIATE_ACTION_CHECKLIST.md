# IMMEDIATE ACTION CHECKLIST
**Created:** 2026-03-10 09:23 UTC  
**Priority:** CRITICAL - Test Now!

---

## ✅ COMPLETED (Just Now)

1. ✅ Implemented `resolve_identity()` in Rust core
2. ✅ Exposed via UniFFI API
3. ✅ Integrated into Android `sendMessage()`
4. ✅ Built Android native libraries
5. ✅ Generated Kotlin bindings
6. ✅ Compiled Android APK
7. ✅ Installed on device (Pixel 6a)

---

## ⏳ IMMEDIATE TESTING REQUIRED (You Do This NOW)

### Test 1: Basic Send
- [ ] Open SCMessenger on Android
- [ ] Select any conversation
- [ ] Type: "Test message 1"
- [ ] Press SEND button
- [ ] **OBSERVE:** Does text field clear?
- [ ] **OBSERVE:** Does message appear?
- [ ] **OBSERVE:** Any error toasts/messages?

### Test 2: Check Logs
```bash
adb logcat -v time | grep "SEND_MSG"
```
- [ ] Look for: `SEND_MSG: Core resolved '...' to publicKey='...'`
- [ ] Check for errors: `Peer not found` or `Core resolution failed`
- [ ] Note any exceptions/stack traces

### Test 3: Multiple Messages
- [ ] Send 5 messages in a row
- [ ] All should go through
- [ ] No UI lag or freezing
- [ ] Messages show as delivered

### Test 4: Cross-Platform
- [ ] Ensure iOS simulator is running
- [ ] Both Android and iOS connected to relay
- [ ] Send from Android → iOS
- [ ] **OBSERVE:** Does iOS receive it?
- [ ] Send from iOS → Android  
- [ ] **OBSERVE:** Does Android receive it?

---

## 📋 REPORT RESULTS

After testing, report:

### If SUCCESS ✅
- "Send is working! Messages deliver successfully."
- Proceed to: Nickname display fix

### If PARTIAL 🟡  
- "Send works sometimes but..."
- Describe: What works? What doesn't?
- Provide: Relevant log excerpt

### If FAILURE ❌
- "Send still not working."
- Provide: Error message from UI
- Provide: Logcat output with SEND_MSG lines
- Describe: Exact symptoms

---

## 🔧 KNOWN REMAINING ISSUES

### 1. Nickname Display
- **Symptom:** Conversations show IDs like `f77690efd...` instead of "John"
- **Priority:** P1
- **Fix:** Use Contact.displayName() in UI
- **Est:** 15-20 lines

### 2. No Block Button
- **Symptom:** Can't block annoying users
- **Priority:** P1  
- **Fix:** Add menu item in ChatScreen
- **Est:** 25-30 lines

### 3. Send Button Lag
- **Symptom:** Text stays in field for a second after send
- **Priority:** P2
- **Fix:** Clear messageText immediately
- **Est:** 2 lines

### 4. iOS Missing Resolver
- **Symptom:** iOS doesn't use new ID resolver yet
- **Priority:** P1
- **Fix:** Integrate resolve_identity() into iOS
- **Est:** 50-75 lines

### 5. iOS Crashes
- **Symptom:** iOS hangs/crashes intermittently
- **Priority:** P2
- **Fix:** Debug with crash logs
- **Est:** TBD

---

## 🚀 NEXT STEPS (After Testing Confirms Success)

1. **Fix Nickname Display**
   - Edit `ConversationsTab.kt`
   - Use `contact.displayName()` or `contact.nickname ?: contact.peerId`
   - Test display shows names

2. **Add Block Button**
   - Edit `ChatScreen.kt` - add IconButton with Block icon
   - Edit `ConversationsViewModel.kt` - add `blockContact()` function
   - Test blocking works

3. **Integrate iOS**
   - Edit iOS MessageViewModel
   - Call `ironCore.resolveIdentity()` before send
   - Rebuild iOS, test

4. **Debug iOS**
   - Collect crash logs
   - Analyze hangs
   - Fix root causes

5. **Update Docs**
   - REMAINING_WORK_TRACKING.md
   - DOCUMENTATION.md
   - Run verify script

---

## 🎯 SUCCESS METRICS

**This is DONE when:**

- [x] Build succeeds (COMPLETE)
- [ ] Send works reliably
- [ ] Nicknames display correctly
- [ ] Contacts persist
- [ ] Cross-platform messaging 100% reliable
- [ ] Block UI functional
- [ ] iOS stable (no crashes)
- [ ] Docs updated

**Current:** 1/8 complete (12.5%)

---

## 📝 QUICK LOG CAPTURE

If you need to capture logs while testing:

```bash
# Method 1: Simple capture
adb logcat -v time > test_$(date +%H%M%S).log

# Then test send, then Ctrl+C

# Method 2: Filter for relevant logs
adb logcat -v time | grep -E "SEND_MSG|resolv|error|scmessenger" > filtered_$(date +%H%M%S).log

# Then test send, then Ctrl+C

# Method 3: Live monitoring
adb logcat -v time -s "MeshRepository:*" "ConversationsViewModel:*"
```

---

## 🆘 IF SOMETHING BREAKS

### Rollback
```bash
cd android
./gradlew clean
./gradlew assembleDebug installDebug
```

### Fresh Install
```bash
# Uninstall app
adb uninstall com.scmessenger.android

# Reinstall
cd android
./gradlew installDebug
```

### Check What's Running
```bash
# See if app is running
adb shell ps | grep scmessenger

# Kill if needed
adb shell am force-stop com.scmessenger.android
```

---

## 💡 TIPS

1. **Test incrementally** - One thing at a time
2. **Capture logs** - Always have logcat running during tests
3. **Note timestamps** - When did error occur?
4. **Try multiple times** - Intermittent issues need multiple attempts
5. **Fresh install if weird** - Sometimes cache causes issues

---

## 📞 REPORTING FORMAT

When you report results, use this format:

```
TEST: Android Send with New Resolver
TIME: [timestamp]
DEVICE: Pixel 6a, Android [version], Cellular

RESULT: [✅ SUCCESS / 🟡 PARTIAL / ❌ FAILURE]

DETAILS:
- Send button: [works/doesn't work]
- Text field: [clears/stays]
- Message appears: [yes/no]
- Delivery status: [delivered/pending/failed]
- Errors: [none/see below]

LOGS:
[paste relevant SEND_MSG lines]

NEXT: [what should we do next?]
```

---

## ⚡ READY? GO TEST!

**Everything is built and installed.**  
**App is on your device.**  
**Just open it and try sending a message!**

**Report back with results and we'll proceed to the next fix!**

---

**End of checklist. Good luck! 🎉**
