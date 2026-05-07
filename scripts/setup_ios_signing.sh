#!/usr/bin/env bash
# iOS signing setup script
# Guides through certificate and provisioning profile export for GitHub Actions
# Validates: Requirements 11.4, 11.10

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🍎 iOS Signing Setup"
echo "===================="

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo -e "${RED}❌ This script must be run on macOS${NC}"
    echo "  iOS signing requires Xcode and Apple Developer tools."
    exit 1
fi

# Check for Xcode
if ! xcode-select -p &>/dev/null; then
    echo -e "${RED}❌ Xcode not found. Please install Xcode from the App Store.${NC}"
    exit 1
fi

echo ""
echo "📋 Prerequisites:"
echo "  1. Apple Developer Account (paid membership)"
echo "  2. Xcode installed"
echo "  3. App registered in App Store Connect"
echo "  4. Distribution certificate created"
echo "  5. Provisioning profile for distribution"
echo ""

read -p "Have you completed the prerequisites? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Please complete prerequisites first:"
    echo "  1. https://developer.apple.com/programs/"
    echo "  2. Create app in App Store Connect"
    echo "  3. Generate distribution certificate in Xcode"
    echo "  4. Create provisioning profile"
    exit 0
fi

echo ""
echo "🔐 Exporting Signing Assets..."
echo "=============================="

# Create output directory
OUTPUT_DIR="ios_signing_assets"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "1. Exporting Distribution Certificate"
echo "-------------------------------------"

# Find distribution certificate
echo "Looking for distribution certificates in Keychain..."
CERTIFICATES=$(security find-identity -p codesigning -v | grep "Distribution" | grep -v "CSSMERR_TP_CERT_REVOKED" || true)

if [ -z "$CERTIFICATES" ]; then
    echo -e "${YELLOW}⚠  No distribution certificates found.${NC}"
    echo "Please create one in Xcode:"
    echo "  1. Open Xcode → Preferences → Accounts"
    echo "  2. Select your team → Manage Certificates"
    echo "  3. Click '+' → Apple Distribution"
    echo "  4. Download and install the certificate"
    exit 1
fi

echo "Available certificates:"
echo "$CERTIFICATES"
echo ""

# Get certificate SHA-1 hash
read -p "Enter the SHA-1 hash of your distribution certificate: " CERT_HASH

# Export certificate as .p12
CERTIFICATE_FILE="$OUTPUT_DIR/distribution_certificate.p12"
echo ""
echo "Exporting certificate to: $CERTIFICATE_FILE"
read -sp "Enter certificate export password: " CERT_PASSWORD
echo

security export -k ~/Library/Keychains/login.keychain-db \
    -t certs -f pkcs12 -P "$CERT_PASSWORD" \
    -o "$CERTIFICATE_FILE" \
    | grep "$CERT_HASH" || true

if [ ! -f "$CERTIFICATE_FILE" ]; then
    echo -e "${RED}❌ Failed to export certificate${NC}"
    echo "Try exporting manually from Keychain Access:"
    echo "  1. Open Keychain Access"
    echo "  2. Find 'Apple Distribution: ...' certificate"
    echo "  3. Right-click → Export"
    echo "  4. Save as .p12 with password"
    exit 1
fi

echo -e "${GREEN}✓ Certificate exported${NC}"

echo ""
echo "2. Exporting Provisioning Profile"
echo "----------------------------------"

# Find provisioning profiles
PROFILES_DIR=~/Library/MobileDevice/Provisioning\ Profiles
if [ ! -d "$PROFILES_DIR" ]; then
    echo -e "${YELLOW}⚠  No provisioning profiles directory found${NC}"
    echo "Please download profiles in Xcode:"
    echo "  1. Open Xcode → Preferences → Accounts"
    echo "  2. Select your team → Download Manual Profiles"
    exit 1
fi

echo "Available provisioning profiles:"
find "$PROFILES_DIR" -name "*.mobileprovision" -exec sh -c 'echo "  - $(basename {}) ($(grep -a "<string>" {} | head -1 | sed "s/.*<string>//;s/<\\/string>.*//"))"' \;

echo ""
read -p "Enter the filename of your distribution provisioning profile: " PROFILE_NAME

PROFILE_SRC="$PROFILES_DIR/$PROFILE_NAME"
PROFILE_DST="$OUTPUT_DIR/distribution_profile.mobileprovision"

if [ ! -f "$PROFILE_SRC" ]; then
    echo -e "${RED}❌ Profile not found: $PROFILE_NAME${NC}"
    echo "Available profiles:"
    ls -la "$PROFILES_DIR"/*.mobileprovision | awk '{print "  - " $9}'
    exit 1
fi

cp "$PROFILE_SRC" "$PROFILE_DST"
echo -e "${GREEN}✓ Provisioning profile copied${NC}"

echo ""
echo "3. Preparing GitHub Secrets Configuration"
echo "-----------------------------------------"

# Base64 encode files
CERTIFICATE_BASE64=$(base64 -i "$CERTIFICATE_FILE")
PROFILE_BASE64=$(base64 -i "$PROFILE_DST")

# Create configuration file
CONFIG_FILE="$OUTPUT_DIR/ios_signing_config.txt"
cat > "$CONFIG_FILE" << EOF
# iOS Signing Configuration
# =========================
# Save these values in GitHub Secrets:
#
# 1. Go to your GitHub repository
# 2. Click Settings → Secrets and variables → Actions
# 3. Click "New repository secret" for each value below

# Base64-encoded distribution certificate (.p12)
IOS_CERTIFICATE_BASE64=$CERTIFICATE_BASE64

# Certificate export password
IOS_CERTIFICATE_PASSWORD=$CERT_PASSWORD

# Base64-encoded provisioning profile
IOS_PROVISIONING_PROFILE_BASE64=$PROFILE_BASE64

# GitHub Actions Workflow Configuration:
# =====================================
# Add to your release workflow:
#
# - name: Import certificates
#   env:
#     CERTIFICATE_BASE64: \${{ secrets.IOS_CERTIFICATE_BASE64 }}
#     CERTIFICATE_PASSWORD: \${{ secrets.IOS_CERTIFICATE_PASSWORD }}
#     PROVISIONING_PROFILE_BASE64: \${{ secrets.IOS_PROVISIONING_PROFILE_BASE64 }}
#   run: |
#     # Import certificate to keychain
#     echo "\$CERTIFICATE_BASE64" | base64 -d > certificate.p12
#     security create-keychain -p "" build.keychain
#     security import certificate.p12 -k build.keychain -P "\$CERTIFICATE_PASSWORD" -T /usr/bin/codesign
#     security set-key-partition-list -S apple-tool:,apple: -s -k "" build.keychain
#     # Install provisioning profile
#     echo "\$PROVISIONING_PROFILE_BASE64" | base64 -d > profile.mobileprovision
#     mkdir -p ~/Library/MobileDevice/Provisioning\ Profiles
#     cp profile.mobileprovision ~/Library/MobileDevice/Provisioning\ Profiles/

# ExportOptions.plist Configuration:
# =================================
# Create iOS/ExportOptions.plist:
#
# <?xml version="1.0" encoding="UTF-8"?>
# <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
# <plist version="1.0">
# <dict>
#     <key>method</key>
#     <string>app-store</string>
#     <key>teamID</key>
#     <string>YOUR_TEAM_ID</string>
#     <key>uploadBitcode</key>
#     <true/>
#     <key>uploadSymbols</key>
#     <true/>
#     <key>provisioningProfiles</key>
#     <dict>
#         <key>com.yourcompany.scmessenger</key>
#         <string>YOUR_PROVISIONING_PROFILE_NAME</string>
#     </dict>
# </dict>
# </plist>

# Xcode Project Configuration:
# ===========================
# Ensure your Xcode project has:
# 1. Correct bundle identifier
# 2. App Store distribution configuration
# 3. Automatic code signing disabled
# 4. Manual signing with correct profile

# Troubleshooting:
# ===============
# Common issues and solutions:
# 1. "Code signing is required" - Check certificate import
# 2. "Provisioning profile not found" - Verify profile name in ExportOptions.plist
# 3. "Team ID mismatch" - Update teamID in ExportOptions.plist
# 4. "Certificate expired" - Renew in Apple Developer portal
EOF

echo -e "${GREEN}✓ Configuration saved to: $CONFIG_FILE${NC}"

# Create backup instructions
BACKUP_FILE="$OUTPUT_DIR/ios_signing_backup_instructions.txt"
cat > "$BACKUP_FILE" << EOF
# iOS Signing Assets Backup Instructions
# ======================================

## Critical: Backup Your Signing Assets!
These files are required to publish updates to the App Store.
If you lose them, you cannot update your app.

## What to Backup:
1. **Distribution Certificate:** $CERTIFICATE_FILE
2. **Certificate Password:** $CERT_PASSWORD
3. **Provisioning Profile:** $PROFILE_DST
4. **Apple Developer Account credentials**

## Where to Store Backups:
✅ **Secure cloud storage** (encrypted):
   - iCloud Keychain (for certificates)
   - Google Drive with encryption
   - 1Password/Dashlane for passwords

✅ **Password manager:**
   - Store certificate password
   - Attach .p12 and .mobileprovision files

✅ **Offline storage:**
   - Encrypted USB drive
   - Time Machine backup (encrypted)
   - External hard drive

❌ **DO NOT store in:**
   - Git repository
   - Unencrypted email
   - Public cloud without encryption
   - Shared folders without access control

## Certificate Expiration:
- Distribution certificates expire after 1 year
- Provisioning profiles expire after 1 year
- Set calendar reminders for renewal

## Renewal Process:
1. **90 days before expiration:**
   - Check Apple Developer portal
   - Renew certificate if needed

2. **60 days before expiration:**
   - Generate new certificate in Xcode
   - Download new provisioning profile
   - Update GitHub Secrets

3. **30 days before expiration:**
   - Test new signing configuration
   - Deploy test build

## Team Management:
- Designate 2+ team members with access
- Use Apple Developer team roles appropriately
- Remove access when team members leave

## Emergency Recovery:
If you lose signing assets:
1. Generate new certificate in Apple Developer portal
2. Create new provisioning profile
3. Update GitHub Secrets
4. Submit new app version (users keep data)
EOF

echo -e "${GREEN}✓ Backup instructions saved to: $BACKUP_FILE${NC}"

echo ""
echo -e "${GREEN}✅ iOS signing setup complete!${NC}"
echo ""
echo "📋 Next steps:"
echo "  1. Review configuration: cat $CONFIG_FILE"
echo "  2. Configure GitHub Secrets with values from $CONFIG_FILE"
echo "  3. Create iOS/ExportOptions.plist file"
echo "  4. Update Xcode project settings"
echo "  5. Test release build locally"
echo "  6. Secure backup: Follow instructions in $BACKUP_FILE"
echo ""
echo "⚠️  Important:"
echo "  - Certificates expire yearly - set renewal reminders"
echo "  - Keep $OUTPUT_DIR/ contents secure"
echo "  - Do NOT commit signing assets to git"
echo "  - Test App Store submission process"
echo ""
echo "📁 Generated files are in: $OUTPUT_DIR/"