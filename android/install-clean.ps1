# SCMessenger Android Clean Build & Install Script for Windows
# Run this from a PowerShell terminal inside the android/ directory.

Set-Location $PSScriptRoot

$AppId = "com.scmessenger.android"

Write-Host "== SCMessenger Android Clean Install ==" -ForegroundColor Green
Write-Host ""

# 1. Check for adb
if (!(Get-Command adb -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] adb is required but not found in PATH." -ForegroundColor Red
    Write-Host "Please install Android Studio or the Command Line Tools and ensure the platform-tools folder is in your PATH." -ForegroundColor Yellow
    Exit 1
}

# 2. Check for connected devices
$devicesOutput = adb devices
$devices = $devicesOutput | Select-String -Pattern "\bdevice\b"

if ($devices.Count -eq 0) {
    Write-Host "No active adb device found, checking for wireless adb..." -ForegroundColor Yellow
    # Trigger an mDNS connect check if mdns services are visible
    $mdnsEndpoints = adb mdns services 2>$null | Select-String -Pattern "_adb-tls-connect\._tcp"
    foreach ($line in $mdnsEndpoints) {
        $endpoint = ($line -split "\s+")[-1]
        if ($endpoint) {
            Write-Host "Attempting wireless connect to $endpoint..." -ForegroundColor Gray
            adb connect $endpoint | Out-Null
        }
    }
    $devicesOutput = adb devices
    $devices = $devicesOutput | Select-String -Pattern "\bdevice\b"
}

if ($devices.Count -eq 0) {
    Write-Host "[ERROR] No connected Android devices detected." -ForegroundColor Red
    Write-Host "Please connect your phone via USB, enable Developer Options, and turn on USB Debugging." -ForegroundColor Yellow
    Exit 1
}

# Select the first connected device
$deviceLine = $devices[0].ToString()
$deviceSerial = ($deviceLine -split "\s+")[0]
Write-Host "Connected Device Serial: $deviceSerial" -ForegroundColor Cyan

# 3. Stop Gradle daemons to avoid file locks
Write-Host "`n1) Stopping Gradle daemons..." -ForegroundColor Cyan
./gradlew.bat --stop

# 4. Run clean build and install
Write-Host "`n2) Cleaning and building debug APK..." -ForegroundColor Cyan
./gradlew.bat clean :app:installDebug

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Gradle build or installation failed." -ForegroundColor Red
    Exit 1
}

# 5. Grant Permissions
Write-Host "`n3) Granting permissions..." -ForegroundColor Cyan
$permissions = @(
    "android.permission.ACCESS_FINE_LOCATION",
    "android.permission.ACCESS_COARSE_LOCATION",
    "android.permission.BLUETOOTH_SCAN",
    "android.permission.BLUETOOTH_ADVERTISE",
    "android.permission.BLUETOOTH_CONNECT",
    "android.permission.NEARBY_WIFI_DEVICES",
    "android.permission.POST_NOTIFICATIONS"
)

foreach ($perm in $permissions) {
    Write-Host "   Granting $perm..." -ForegroundColor Gray
    adb -s $deviceSerial shell pm grant $AppId $perm 2>$null
}

Write-Host "`n[OK] Install complete! Launching app..." -ForegroundColor Green
adb -s $deviceSerial shell am start -n "$AppId/.ui.MainActivity"
