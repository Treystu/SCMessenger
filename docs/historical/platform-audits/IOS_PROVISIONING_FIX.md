# iOS Build Fix - Provisioning Profile Expired
**Date**: 2026-03-09
**Issue**: Provisioning profile expired, authentication required
**Status**: ⚠️ REQUIRES MANUAL INTERVENTION

---

## Problem

```
Authentication failed: Your session has expired. Please log in.
Provisioning profile "iOS Team Provisioning Profile: SovereignCommunications.SCMessenger" expired on Mar 9, 2026.
```

---

## Root Cause

- **Provisioning profile expired**: March 9, 2026 (today)
- **Apple ID session expired**: Need to re-authenticate
- **Automatic signing disabled** or failed to renew

---

## Fix Steps (Manual - Required)

### Option 1: Xcode GUI (Recommended)

1. **Open Xcode**:
   ```bash
   open /Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger.xcodeproj
   ```

2. **Sign in to Apple Account**:
   - Xcode → Settings (⌘,)
   - Click "Accounts" tab
   - Click "+" and add your Apple ID
   - Enter credentials when prompted

3. **Manage Signing**:
   - Select project in navigator
   - Select "SCMessenger" target
   - Go to "Signing & Capabilities" tab
   - Check "Automatically manage signing"
   - Select your Team from dropdown
   - Xcode will auto-renew the provisioning profile

4. **Verify**:
   - Check that Status shows: "Ready"
   - Provisioning Profile should show new expiration date
   - No error messages

5. **Build**:
   ```bash
   cd iOS/SCMessenger
   xcodebuild -scheme SCMessenger -configuration Debug -destination 'platform=iOS,id=00008130-001A48DA18EB8D3A' build
   ```

### Option 2: Command Line (Advanced)

If you prefer CLI:

```bash
# 1. List available provisioning profiles
security find-identity -v -p codesigning

# 2. Delete expired profile
rm ~/Library/MobileDevice/Provisioning\ Profiles/*.mobileprovision

# 3. Login to Apple (requires GUI)
open "https://appleid.apple.com"

# 4. Download new profile via Xcode
xcodebuild -downloadAllPlatforms
```

### Option 3: Temporary Workaround (Local Testing Only)

For local device testing without App Store distribution:

1. **Disable signing requirement** (Xcode 14+):
   - Select target → Signing & Capabilities
   - Uncheck "Automatically manage signing"
   - Set Signing Certificate to "Sign to Run Locally"
   - This allows local device testing without valid profile

2. **Or use Debug signing**:
   - Build for Simulator instead of device:
   ```bash
   xcodebuild -scheme SCMessenger -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 15' build
   ```

---

## Expected Outcome

After fixing:
- ✅ Authentication successful
- ✅ New provisioning profile downloaded
- ✅ Expiration date: ~1 year from now (Mar 9, 2027)
- ✅ Build succeeds
- ✅ Can deploy to device

---

## Common Issues

### "Failed to create provisioning profile"
- **Solution**: Check Apple Developer Program membership is active
- **Solution**: Ensure App ID exists at developer.apple.com
- **Solution**: Check device is registered in Apple Developer portal

### "No signing certificate found"
- **Solution**: Install developer certificate from Apple Developer portal
- **Solution**: Or let Xcode create one automatically

### "Team not found"
- **Solution**: Verify Apple ID is added to Xcode Accounts
- **Solution**: Check you're using correct Apple ID for the team

---

## Verification

After renewing, verify with:

```bash
# Check provisioning profile
security cms -D -i ~/Library/MobileDevice/Provisioning\ Profiles/*.mobileprovision | grep -A 2 "ExpirationDate"

# Should show future date, not Mar 9, 2026
```

---

## Prevention

To avoid future expirations:

1. **Enable Automatic Signing** in Xcode
   - Xcode auto-renews profiles before expiration
   - Requires active Apple Developer account

2. **Set Calendar Reminder**
   - Profiles expire yearly
   - Set reminder for ~2 weeks before expiration

3. **Use CI/CD with Fastlane**
   - Automates certificate/profile management
   - Can be configured to auto-renew

---

## Once Fixed

After renewing provisioning profile:

1. **Rebuild Core Framework** (if needed):
   ```bash
   cd /Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger
   ./rebuild_ios_core.sh
   ```

2. **Build iOS App**:
   ```bash
   cd iOS/SCMessenger
   xcodebuild -scheme SCMessenger -configuration Debug \
     -destination 'platform=iOS,id=00008130-001A48DA18EB8D3A' build
   ```

3. **Deploy to Device**:
   - Xcode will install automatically
   - Or use: `xcodebuild -scheme SCMessenger -destination ... install`

---

## Status

**Current**: ⚠️ Provisioning profile expired
**Action Required**: Renew profile via Xcode
**Estimated Time**: 2-5 minutes
**Impact**: Blocks iOS builds until renewed

---

## Notes

- Provisioning profiles expire annually
- Free Apple Developer accounts expire every 7 days
- Paid accounts ($99/year) get 1-year profiles
- Automatic signing handles renewal automatically (if enabled)

**Recommendation**: Use Option 1 (Xcode GUI) - fastest and most reliable
