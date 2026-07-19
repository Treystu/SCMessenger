# SCMessenger Android Build Setup Verification for Windows
# Run this from a PowerShell terminal inside the android/ directory.

Set-Location $PSScriptRoot

Write-Host "=== SCMessenger Android Build Verification ===" -ForegroundColor Green
Write-Host ""

$allOk = $true

# Helper to check commands
function Check-Command($cmd) {
    if (Get-Command $cmd -ErrorAction SilentlyContinue) {
        Write-Host "[OK] $cmd found" -ForegroundColor Green
        return $true
    } else {
        Write-Host "[ERROR] $cmd not found" -ForegroundColor Red
        return $false
    }
}

# Helper to check files
function Check-File($path) {
    if (Test-Path $path -PathType Leaf) {
        Write-Host "[OK] File exists: $path" -ForegroundColor Green
        return $true
    } else {
        Write-Host "[ERROR] File missing: $path" -ForegroundColor Red
        return $false
    }
}

# 1. Rust Toolchain
Write-Host "1. Checking Rust toolchain..." -ForegroundColor Cyan
if (Check-Command "rustc") {
    $ver = rustc --version
    Write-Host "   Version: $ver"
} else {
    $allOk = $false
    Write-Host "   Install from: https://rustup.rs" -ForegroundColor Yellow
}

if (!(Check-Command "cargo")) {
    $allOk = $false
}
Write-Host ""

# 2. cargo-ndk
Write-Host "2. Checking cargo-ndk..." -ForegroundColor Cyan
if (Check-Command "cargo-ndk") {
    $ver = cargo-ndk --version 2>&1
    Write-Host "   Version: $ver"
} else {
    $allOk = $false
    Write-Host "   Install by running: cargo install cargo-ndk" -ForegroundColor Yellow
}
Write-Host ""

# 3. Android Rust Targets
Write-Host "3. Checking Android Rust targets..." -ForegroundColor Cyan
$targets = @("aarch64-linux-android", "x86_64-linux-android")
$targetsOk = $true
$installedTargets = rustup target list | Where-Object { $_ -match "installed" }
foreach ($target in $targets) {
    if ($installedTargets -match $target) {
        Write-Host "   [OK] $target installed" -ForegroundColor Green
    } else {
        Write-Host "   [ERROR] $target not installed" -ForegroundColor Red
        $targetsOk = $false
    }
}
if (!$targetsOk) {
    $allOk = $false
    Write-Host "   Install targets by running: rustup target add aarch64-linux-android x86_64-linux-android" -ForegroundColor Yellow
}
Write-Host ""

# 4. Java JDK
Write-Host "4. Checking Java..." -ForegroundColor Cyan
if (Check-Command "java") {
    $ver = java -version 2>&1 | Select-Object -First 1
    Write-Host "   Version: $ver"
} else {
    $allOk = $false
    Write-Host "   Java 17+ is required. Please install OpenJDK 17." -ForegroundColor Yellow
}
Write-Host ""

# 5. ANDROID_HOME
Write-Host "5. Checking ANDROID_HOME..." -ForegroundColor Cyan
$androidHome = $env:ANDROID_HOME
if ($androidHome -and (Test-Path $androidHome -PathType Container)) {
    Write-Host "[OK] ANDROID_HOME set: $androidHome" -ForegroundColor Green
    $ndkPath = Join-Path $androidHome "ndk\26.1.10909125"
    if (Test-Path $ndkPath -PathType Container) {
        Write-Host "   [OK] NDK 26.1.10909125 installed" -ForegroundColor Green
    } else {
        Write-Host "   [WARNING] NDK 26.1.10909125 not found (Android Studio will download it)" -ForegroundColor Yellow
    }
} else {
    Write-Host "[ERROR] ANDROID_HOME not set or invalid" -ForegroundColor Red
    Write-Host "   Set the ANDROID_HOME environment variable to your Android SDK folder." -ForegroundColor Yellow
    $allOk = $false
}
Write-Host ""

# 6. Project Structure
Write-Host "6. Checking project structure..." -ForegroundColor Cyan
$rootPath = Resolve-Path ".."
$structureOk = $true
$structureOk = $structureOk -and (Check-File (Join-Path $rootPath "core\src\api.udl"))
$structureOk = $structureOk -and (Check-File (Join-Path $rootPath "core\src\bin\gen_kotlin.rs"))
$structureOk = $structureOk -and (Check-File (Join-Path $rootPath "android\app\build.gradle"))
$structureOk = $structureOk -and (Check-File (Join-Path $rootPath "android\gradlew.bat"))
if (!$structureOk) { $allOk = $false }
Write-Host ""

# 7. Bindings generation test
Write-Host "7. Testing UniFFI bindings generation..." -ForegroundColor Cyan
Push-Location (Join-Path $rootPath "core")
$cargoRun = cargo run --bin gen_kotlin --features gen-bindings 2>&1
$bindingsFile = Join-Path $rootPath "core\target\generated-sources\uniffi\kotlin\uniffi\api\api.kt"
if ($LASTEXITCODE -eq 0 -and (Test-Path $bindingsFile -PathType Leaf)) {
    $size = (Get-Item $bindingsFile).Length
    Write-Host "[OK] Bindings generated successfully ($size bytes)" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Bindings generation failed" -ForegroundColor Red
    Write-Host "   Cargo output:" -ForegroundColor Yellow
    $cargoRun | Out-String | Write-Host -ForegroundColor Gray
    $allOk = $false
}
Pop-Location
Write-Host ""

# Final Summary
Write-Host "=== Summary ===" -ForegroundColor Cyan
if ($allOk) {
    Write-Host "[OK] All checks passed! Android build should work." -ForegroundColor Green
    Write-Host "To build, run: cd android; .\gradlew.bat assembleDebug" -ForegroundColor Yellow
} else {
    Write-Host "[ERROR] Some checks failed. Fix the issues above before building." -ForegroundColor Red
}
