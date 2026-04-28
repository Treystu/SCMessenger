# Contact Visibility Debugging - March 10, 2026

## Problem Statement
- 2 iOS Simulators running on same Mac
- 1 Android device on cellular
- All connected to relay nodes
- **None can see each other**

## Current State

### Running Processes
- iPhone 16e: PID 16634 (UDID: E3284273...)
- iPhone 17 Pro: PID 20528 (UDID: F7AAF4C8...)
- Android: Running on cellular

### Observable Symptoms
iPhone 16e trying to send to peer `12D3KooWCitiz...`:
- Core-routed delivery failed: IronCoreError 4
- Relay-circuit retry failed: IronCoreError 4

## Investigation

### Question 1: Why don't iOS sims see each other?
**Same Mac, same network** - Should discover via:
- Multicast DNS (mDNS)
- Network local discovery
- Shared relay

### Question 2: Why don't they see Android via relay?
**All connected to relay** - Should get peer lists from relay

### Question 3: What is IronCoreError 4?
Need to check error mapping

## Debugging Steps

1. Get own peer IDs for all 3 devices
2. Check what peers each device sees
3. Verify relay connections
4. Check if peers are being broadcast
5. Examine contact lists vs discovered peers


## ROOT CAUSE FOUND

### Android Sees:
- Only `12D3KooWDWQm...` (old peer from previous session via relay circuit)
- **Does NOT see either iOS simulator**

### iOS Simulators Status:
- Both launched but **no mesh service logs**
- No "Peer identified" or "Peer discovered" events
- **Likely not initialized with identity yet**

## The Problem

**iOS simulators need to:**
1. Create an identity (tap "Generate Identity" in onboarding)
2. Start mesh service
3. Connect to relay
4. Broadcast their presence

**Without identity creation, they are just UI shells with no mesh networking active.**

## Solution

### For each iOS Simulator:
1. Tap on simulator window to focus it
2. Complete onboarding: Create identity with a nickname
3. Wait for mesh service to start
4. Check logs for "Peer discovered" events

### Expected Behavior After Identity Creation:
- iOS Sim 1 will see: iOS Sim 2, Android (via relay)
- iOS Sim 2 will see: iOS Sim 1, Android (via relay)
- Android will see: iOS Sim 1, iOS Sim 2 (via relay)

## Verification Commands

After creating identities on both iOS sims:

```bash
# Check iOS 16e peers
xcrun simctl spawn E3284273 log show --last 30s \
  --predicate 'process == "SCMessenger"' --style compact | \
  grep "Peer.*identified"

# Check iOS 17 Pro peers
xcrun simctl spawn F7AAF4C8 log show --last 30s \
  --predicate 'process == "SCMessenger"' --style compact | \
  grep "Peer.*identified"

# Check Android peers
adb logcat -d | grep "IdentityDiscovered" | tail -10
```

## Why This Wasn't Obvious

1. Apps launched successfully (no crash)
2. UI showed up (looked normal)
3. But mesh service requires identity to start
4. Without identity = no networking = invisible to other peers

## Next Steps

1. **User:** Create identities on both iOS sims
2. **User:** Wait 10 seconds for peer discovery
3. **Test:** Try sending messages between all 3 devices
4. **Verify:** Check delivery states work correctly

