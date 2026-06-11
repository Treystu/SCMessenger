#!/usr/bin/env bash
# Android signing setup script
# Generates release keystore and prepares GitHub Secrets configuration
# Validates: Requirements 11.1, 11.10, 11.11

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔐 Android Signing Setup"
echo "========================"

# Check if keytool is available
if ! command -v keytool &> /dev/null; then
    echo -e "${RED}❌ keytool not found. Please install Java JDK.${NC}"
    echo "  On Ubuntu/Debian: sudo apt install openjdk-11-jdk"
    echo "  On macOS: brew install openjdk"
    echo "  On Windows: Download from https://adoptium.net/"
    exit 1
fi

# Configuration
KEYSTORE_NAME="release.keystore"
KEY_ALIAS="scmessenger"
KEY_VALIDITY_DAYS=10000  # ~27 years
KEY_SIZE=2048
KEY_ALGORITHM="RSA"

echo ""
echo "📋 Configuration:"
echo "  - Keystore: $KEYSTORE_NAME"
echo "  - Key alias: $KEY_ALIAS"
echo "  - Validity: $KEY_VALIDITY_DAYS days"
echo "  - Key size: $KEY_SIZE bits"
echo "  - Algorithm: $KEY_ALGORITHM"
echo ""

# Check if keystore already exists
if [ -f "$KEYSTORE_NAME" ]; then
    echo -e "${YELLOW}⚠  Keystore '$KEYSTORE_NAME' already exists.${NC}"
    read -p "Do you want to overwrite it? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
    rm -f "$KEYSTORE_NAME"
fi

# Generate keystore
echo "🔑 Generating keystore..."
keytool -genkeypair \
    -v \
    -keystore "$KEYSTORE_NAME" \
    -alias "$KEY_ALIAS" \
    -keyalg "$KEY_ALGORITHM" \
    -keysize "$KEY_SIZE" \
    -validity "$KEY_VALIDITY_DAYS" \
    -storetype PKCS12 \
    -dname "CN=SCMessenger, OU=Development, O=SCMessenger, L=San Francisco, S=California, C=US" \
    -storepass "temporary_password" \
    -keypass "temporary_password"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Keystore generated successfully${NC}"
else
    echo -e "${RED}❌ Failed to generate keystore${NC}"
    exit 1
fi

# Verify keystore
echo ""
echo "🔍 Verifying keystore..."
keytool -list -v \
    -keystore "$KEYSTORE_NAME" \
    -alias "$KEY_ALIAS" \
    -storepass "temporary_password" 2>/dev/null | grep -E "(Alias name|Creation date|Entry type|Key size)"

# Generate random passwords
echo ""
echo "🎲 Generating secure passwords..."
KEYSTORE_PASSWORD=$(openssl rand -base64 32)
KEY_PASSWORD=$(openssl rand -base64 32)

echo -e "${GREEN}✓ Passwords generated${NC}"

# Update keystore with secure passwords
echo ""
echo "🔐 Updating keystore passwords..."
keytool -keypasswd \
    -keystore "$KEYSTORE_NAME" \
    -alias "$KEY_ALIAS" \
    -storepass "temporary_password" \
    -keypass "temporary_password" \
    -new "$KEY_PASSWORD"

keytool -storepasswd \
    -keystore "$KEYSTORE_NAME" \
    -storepass "temporary_password" \
    -new "$KEYSTORE_PASSWORD"

echo -e "${GREEN}✓ Keystore passwords updated${NC}"

# Base64 encode keystore for GitHub Secrets
echo ""
echo "📦 Preparing GitHub Secrets configuration..."
KEYSTORE_BASE64=$(base64 -w 0 "$KEYSTORE_NAME")

# Create configuration file
CONFIG_FILE="android_signing_config.txt"
cat > "$CONFIG_FILE" << EOF
# Android Signing Configuration
# =============================
# Save these values in GitHub Secrets:
#
# 1. Go to your GitHub repository
# 2. Click Settings → Secrets and variables → Actions
# 3. Click "New repository secret" for each value below

# Base64-encoded keystore file
ANDROID_KEYSTORE_BASE64=$KEYSTORE_BASE64

# Keystore password
KEYSTORE_PASSWORD=$KEYSTORE_PASSWORD

# Key alias (use the same as below)
KEYSTORE_ALIAS=$KEY_ALIAS

# Key password
KEY_PASSWORD=$KEY_PASSWORD

# Android build.gradle configuration:
# Add to android/app/build.gradle:
#
# android {
#     signingConfigs {
#         release {
#             storeFile file("release.keystore")
#             storePassword System.getenv("KEYSTORE_PASSWORD")
#             keyAlias System.getenv("KEYSTORE_ALIAS")
#             keyPassword System.getenv("KEY_PASSWORD")
#         }
#     }
#     buildTypes {
#         release {
#             signingConfig signingConfigs.release
#             minifyEnabled true
#             proguardFiles getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro"
#         }
#     }
# }

# GitHub Actions workflow configuration:
# Add to your release workflow:
#
# - name: Decode keystore
#   run: |
#     echo "\${{ secrets.ANDROID_KEYSTORE_BASE64 }}" | base64 -d > release.keystore
# - name: Build Release
#   env:
#     KEYSTORE_FILE: release.keystore
#     KEYSTORE_PASSWORD: \${{ secrets.KEYSTORE_PASSWORD }}
#     KEYSTORE_ALIAS: \${{ secrets.KEYSTORE_ALIAS }}
#     KEY_PASSWORD: \${{ secrets.KEY_PASSWORD }}
#   run: |
#     cd android
#     ./gradlew assembleRelease bundleRelease

# Backup Instructions:
# ===================
# 1. Keep a secure backup of '$KEYSTORE_NAME'
# 2. Store passwords in a password manager
# 3. Do NOT commit keystore to git
# 4. Rotate keys every 2 years

# Key Rotation:
# =============
# 1. Generate new keystore with this script
# 2. Update GitHub Secrets with new values
# 3. Update app signing in Google Play Console
# 4. Maintain old keystore for app updates
EOF

echo -e "${GREEN}✓ Configuration saved to: $CONFIG_FILE${NC}"

# Create backup instructions
BACKUP_FILE="android_signing_backup_instructions.txt"
cat > "$BACKUP_FILE" << EOF
# Android Signing Key Backup Instructions
# =======================================

## Critical: Backup Your Signing Key!
The keystore file '$KEYSTORE_NAME' is required to publish updates to your app.
If you lose it, you cannot update your app on Google Play Store.

## What to Backup:
1. **Keystore file:** $KEYSTORE_NAME
2. **Passwords:** 
   - Keystore password: $KEYSTORE_PASSWORD
   - Key password: $KEY_PASSWORD
   - Key alias: $KEY_ALIAS

## Where to Store Backups:
✅ **Secure cloud storage** (encrypted):
   - Google Drive with encryption
   - Dropbox with Boxcryptor
   - OneDrive with encryption

✅ **Password manager:**
   - 1Password, LastPass, Bitwarden
   - Store passwords and keystore as attachment

✅ **Offline storage:**
   - Encrypted USB drive
   - External hard drive
   - Printed QR code of base64 keystore (secure location)

❌ **DO NOT store in:**
   - Git repository
   - Unencrypted email
   - Public cloud without encryption
   - Shared folders without access control

## Recovery Process:
If you lose access to your signing key:
1. Generate new keystore with setup_android_signing.sh
2. Create new app in Google Play Console
3. Users must uninstall old version and install new version
4. Ratings and reviews are lost

## Key Rotation Schedule:
- Every 2 years for security best practices
- When team members leave
- If you suspect key compromise

## Emergency Contact:
Designate at least 2 team members who have access to the backup.
EOF

echo -e "${GREEN}✓ Backup instructions saved to: $BACKUP_FILE${NC}"

echo ""
echo -e "${GREEN}✅ Android signing setup complete!${NC}"
echo ""
echo "📋 Next steps:"
echo "  1. Review configuration: cat $CONFIG_FILE"
echo "  2. Configure GitHub Secrets with values from $CONFIG_FILE"
echo "  3. Secure backup: Follow instructions in $BACKUP_FILE"
echo "  4. Update android/app/build.gradle with signing configuration"
echo "  5. Test release build locally"
echo ""
echo "⚠️  Important:"
echo "  - Keep $KEYSTORE_NAME and passwords secure"
echo "  - Do NOT commit keystore to git"
echo "  - Create multiple backups in different locations"
echo "  - Test release build before deploying to production"