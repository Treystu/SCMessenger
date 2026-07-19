# SCMessenger — Windows Android Quick Start Guide

This guide will walk you through setting up your Windows computer and installing SCMessenger on your Android phone. Follow these steps sequentially.

---

## Step 1: Install Android Studio
SCMessenger requires the Android development tools to compile and install on your phone.
1. Download **Android Studio**: [Download here](https://developer.android.com/studio)
2. Run the downloaded installer and accept all default settings.
3. Once installed, open Android Studio. During the initial setup wizard:
   - Choose **Standard** installation.
   - Let it download and install the required Android SDK, platform-tools, and emulator packages.
4. **Critical**: Keep Android Studio open during your first setup.

---

## Step 2: Install Rust
SCMessenger's secure core is written in Rust. You need the Rust compiler installed on your computer.
1. Download **Rustup** (the Rust installer): [Download here](https://rustup.rs/) (Click the `rustup-init.exe` link for 64-bit Windows).
2. Run `rustup-init.exe`.
3. A command prompt window will open. Type `1` and press **Enter** to install with default settings.
4. Once completed, close the command window.

---

## Step 3: Enable USB Debugging on Your Android Phone
To copy the app from your computer to your phone, you must allow "USB Debugging."
1. Open your phone's **Settings**.
2. Scroll to the bottom and tap **About phone** (or **About device**).
3. Find **Build number** (usually under "Software information") and tap it rapidly **7 times**. You will see a popup saying *"You are now a developer!"*
4. Go back to the main Settings screen and tap the new **Developer options** menu (usually under "System" or at the bottom of settings).
5. Scroll down and turn ON **USB debugging**.
6. Connect your phone to your computer using a USB cable.
7. A prompt will appear on your phone asking to *“Allow USB debugging?”* Check the box for **"Always allow from this computer"** and tap **Allow**.

---

## Step 4: Verify Your Setup
We have included a script to check if everything is configured correctly.
1. Open the SCMessenger folder on your computer.
2. Open the `android` folder.
3. Double-click the file named **`verify-build-setup.bat`**.
4. A window will open showing the diagnostics.
   - If everything is `[OK]`, you are ready to install!
   - If there is an error, follow the instructions printed in yellow on the screen to resolve it.

---

## Step 5: Install SCMessenger
To compile the secure core, build the Android app, and install it on your phone:
1. Make sure your phone is still connected to the computer via USB and unlocked.
2. Double-click the file named **`install-clean.bat`** in the `android` folder.
3. This script will automatically:
   - Stop background build locks.
   - Compile the Rust secure core and compile the Android app.
   - Install the app directly onto your connected phone.
   - Grant all required permissions (Bluetooth, Location, and Local Network Sharing) so mesh discovery starts immediately.
   - Launch the SCMessenger app on your phone.
4. Once the process completes, the window will show `[OK] Install complete!` and you can press any key to close it.

---

## Step 6: Connect to the Mesh (Lucas <-> Josh Alpha Test)
The application is pre-configured to automatically connect to the cloud relay (`100.56.248.69:9001`) in the background at startup.
1. Open SCMessenger on your phone.
2. The app will automatically bootstrap and connect to the mesh network.
3. If you ever need to manually connect or re-add the bootstrap address, go to the **Join Mesh** screen, type in the address below, and tap **Join**:
   ```
   /ip4/100.56.248.69/tcp/9001
   ```
