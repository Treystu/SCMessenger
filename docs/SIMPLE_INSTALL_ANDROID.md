# SCMessenger — Simple Android Installation Guide

This guide describes how to quickly install SCMessenger on an Android phone using a pre-built APK file from a Windows or Mac/Linux computer.

---

## Requirements

1. An **Android Phone**.
2. A **Computer** (Windows laptop, Mac, or Linux).
3. A **USB Cable** to connect the phone to the computer.
4. The **`app-debug.apk`** file (or `install-apk.bat` / platform package).

---

## Step 1: Enable USB Debugging on the Phone (One-Time Setup)

1. On your Android phone, open **Settings**.
2. Scroll to the bottom and tap **About Phone** (or **Software Information**).
3. Find **Build Number** and tap it quickly **7 times** until a popup says *"You are now a developer!"*.
4. Go back to **Settings** -> **System** (or main menu) -> **Developer Options**.
5. Turn **ON** **USB Debugging**.

---

## Step 2: Connect Phone to Computer

1. Plug your Android phone into the computer using a USB cable.
2. Unlock your phone screen.
3. When a prompt appears on the phone screen asking *"Allow USB Debugging?"*:
   - Check the box for **"Always allow from this computer"**.
   - Tap **Allow**.

---

## Step 3: Install the App

### Option A: Windows (Automated 1-Click Script)

1. Ensure `app-debug.apk` and `install-apk.bat` are in the same folder on your Windows computer.
2. Double-click **`install-apk.bat`**.
3. The script will automatically:
   - Detect your connected phone.
   - Install SCMessenger.
   - Automatically grant required location, Bluetooth, and network permissions.
   - Launch SCMessenger on your phone.

### Option B: Command Line (Windows / Mac / Linux via ADB)

If you have `adb` installed:
```bash
# 1. Verify device connection
adb devices

# 2. Install the pre-built APK
adb install -r app-debug.apk

# 3. Grant initial network and Bluetooth permissions
adb shell pm grant com.scmessenger.android android.permission.ACCESS_FINE_LOCATION
adb shell pm grant com.scmessenger.android android.permission.ACCESS_COARSE_LOCATION
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_SCAN
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_ADVERTISE
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_CONNECT
adb shell pm grant com.scmessenger.android android.permission.NEARBY_WIFI_DEVICES
adb shell pm grant com.scmessenger.android android.permission.POST_NOTIFICATIONS

# 4. Launch the app
adb shell am start -n "com.scmessenger.android/.ui.MainActivity"
```

---

## Step 4: Open & Connect

1. Open **SCMessenger** on the phone.
2. Grant any remaining system prompts (Bluetooth / Location / Local Network) when prompted inside the app.
3. SCMessenger automatically connects to the network relay (`100.56.248.69:9001`) over cellular data or Wi-Fi.
