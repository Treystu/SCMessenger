# Alpha Auto-Update Plan (CI/CD via GitHub Actions)

To achieve a "hands-off" testing experience where your iOS and Android devices automatically receive the latest builds when you push to GitHub, we need to set up Continuous Integration/Continuous Deployment (CI/CD) pipelines.

Because we cannot bypass Apple and Google's OS-level restrictions regarding how apps install code, we will use their official beta-testing platforms: **TestFlight (iOS)** and **Google Play Internal Testing (Android)**.

Both of these platforms support **Automatic Updates** on the end-user's device. Once configured, you push code to GitHub, GitHub Actions builds it, and the platforms push it to the phones.

---

## 1. iOS: TestFlight Auto-Updates

TestFlight is the only Apple-sanctioned way to have an app update itself in the background without user intervention during the Alpha phase.

### Requirements:

1. **Apple Developer Program Account**: Required for App Store Connect ($99/yr).
2. **App Store Connect API Key**: Needed so GitHub Actions can upload the build securely.
3. **Provisioning Profiles & Certificates**: Standard iOS code signing setup.

### The Pipeline Flow (GitHub Actions + Fastlane):

1. **Trigger**: You push a commit to the `main` branch (or create a GitHub Release).
2. **Build**: GitHub Actions spins up a `macos-latest` runner.
3. **Code Signing**: It pulls your certificates (Securely using Fastlane Match or GitHub Secrets).
4. **Compile**: It builds SCMessenger (`xcodebuild` / fastlane `gym`).
5. **Upload**: It uploads the `.ipa` to TestFlight using the App Store Connect API.

### The End-User Experience:

1. You install the "TestFlight" app from the App Store on your iPhone.
2. You invite your Apple ID to the internal testers group.
3. You download SCMessenger from TestFlight.
4. **The Magic Step**: Inside TestFlight, you toggle **"Automatic Updates"** ON for SCMessenger.
5. _Result_: Every time GitHub Actions finishes an upload, iOS will invisibly download and install the new version in the background.

---

## 2. Android: Google Play Internal Testing

For Android, we have two good options: Firebase App Distribution or Google Play Console. To get true "background auto-updates" akin to the App Store, **Google Play Internal Testing** is the best route.

### Requirements:

1. **Google Play Developer Account**: ($25 one-time fee).
2. **Google Cloud Service Account JSON**: Needed for GitHub Actions to authenticate with the Play Developer API.
3. **Keystore File**: Used to sign your release APK/AAB.

### The Pipeline Flow (GitHub Actions):

1. **Trigger**: Push to `main` branch.
2. **Build**: GitHub Actions spins up an `ubuntu-latest` runner.
3. **Rust/NDK Setup**: It installs the Android NDK and Rust targets for Android (`aarch64-linux-android`, etc.).
4. **Compile**: Builds the Android project (`./gradlew assembleRelease` or `bundleRelease`).
5. **Upload**: Uses a GitHub Action (like `r0adkll/upload-google-play`) to push the `.aab` file to the "Internal Testing" track on Google Play.

### The End-User Experience:

1. You add your Google account to the internal testers list in the Play Console.
2. You opt-in via a special web link provided by the Play Console.
3. You download the app through the regular Google Play Store app on your phone.
4. **The Magic Step**: Just like standard apps, as long as "Auto-update apps" is enabled in your Google Play Store settings, new updates pushed to the Internal Testing track will automatically install in the background.

_(Alternative: Firebase App Distribution is easier to set up, but users receive an email/notification and must manually tap "Update" in the App Tester app. It does not auto-install in the background)._

---

## Implementation Steps for Alpha

When we are ready to build this, we will execute the following steps:

1. **Create `fastlane` configuration** in the repository for both iOS and Android.
2. **Generate all API Keys/Secrets** (App Store Connect API, Google Play Service Account, Keystores).
3. **Add Secrets to GitHub Repository**.
4. **Write `.github/workflows/deploy-ios-alpha.yml`**.
5. **Write `.github/workflows/deploy-android-alpha.yml`**.
6. **Perform Initial Manual Uploads** (Both Apple and Google require the very first build to be uploaded manually via their web consoles/apps before APIs can take over).

By following this architecture, pushing code to the `alpha` or `main` branch will seamlessly result in the apps automatically updating on your test devices within 15-30 minutes.
