# SCMessenger — Simple iOS Installation Guide

This guide describes how to install a pre-built SCMessenger iOS app binary onto an iPhone using a Mac.

---

## Requirements

1. An **iPhone** (running iOS 17.0 or newer).
2. A **Mac Computer** (with Xcode or Apple Configurator installed).
3. A **Lightning or USB-C Cable** to connect the iPhone to the Mac.
4. The pre-built **`SCMessenger.app`** or signed **`SCMessenger.ipa`** file.

---

## Step 1: Enable Developer Mode on iOS (iOS 17+)

1. On your iPhone, open **Settings**.
2. Tap **Privacy & Security**.
3. Scroll to the bottom and tap **Developer Mode**.
4. Toggle **Developer Mode** to **ON**.
5. Restart your iPhone when prompted.
6. After rebooting and unlocking your iPhone, tap **Turn On** on the popup screen and enter your passcode.

---

## Step 2: Connect iPhone to Mac

1. Connect your iPhone to the Mac using a USB cable.
2. Unlock your iPhone screen.
3. If prompted on the iPhone, tap **Trust This Computer** and enter your passcode.

---

## Step 3: Install the Pre-built App

### Option A: Using Xcode (GUI)

1. Open **Xcode** on your Mac.
2. Go to **Window** -> **Devices and Simulators** (or press `Cmd + Shift + 2`).
3. Select your connected iPhone under **Devices** on the left panel.
4. Under **Installed Apps**, drag and drop the pre-built **`SCMessenger.app`** or **`SCMessenger.ipa`** file into the list (or click the `+` icon and choose the file).
5. The installation will complete in a few seconds.

### Option B: Using Apple Configurator (GUI)

1. Download/open **Apple Configurator** from the Mac App Store.
2. Connect your iPhone.
3. Drag and drop the `.ipa` binary file directly onto the iPhone icon in Apple Configurator.
4. Confirm installation.

### Option C: Using Command Line (via `devicectl` / `ios_deploy`)

If using command-line tools on Mac:
```bash
# 1. List connected device UDID
xcrun devicectl list devices

# 2. Install pre-built app bundle
xcrun devicectl device install app --device <DEVICE_UDID> /path/to/SCMessenger.app
```
Or via the repo's helper script:
```bash
export DEVICE_UDID="<YOUR_DEVICE_UDID>"
bash iOS/install-device.sh
```

---

## Step 4: Open & Trust Application

1. If prompted when opening SCMessenger (*"Untrusted Developer"*):
   - Go to iPhone **Settings** -> **General** -> **VPN & Device Management**.
   - Tap your Developer Account / Certificate name under **Developer App**.
   - Tap **Trust "[Developer Name]"** -> **Trust**.
2. Launch **SCMessenger** from your home screen.
3. Grant **Bluetooth**, **Local Network**, and **Notification** permissions when prompted.
