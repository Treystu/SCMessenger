#!/bin/bash
set -e

echo "Building mobile core for iOS..."
cd mobile
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim

cd ..
echo "Generating Swift bindings..."
cd core
cargo run --bin gen_swift --features gen-bindings
cd ..

echo "Creating XCFramework..."
rm -rf SCMessengerCore.xcframework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libscmessenger_mobile.a \
  -headers core/target/generated-sources/uniffi/swift/ \
  -library target/aarch64-apple-ios-sim/release/libscmessenger_mobile.a \
  -headers core/target/generated-sources/uniffi/swift/ \
  -output SCMessengerCore.xcframework

rm -rf iOS/SCMessengerCore.xcframework
cp -R SCMessengerCore.xcframework iOS/SCMessengerCore.xcframework

echo "Copying generated Swift bindings to iOS project..."
# UniFFI 0.31 uses different filenames
cp core/target/generated-sources/uniffi/swift/SCMessengerCore.swift iOS/SCMessenger/SCMessenger/Generated/api.swift
cp core/target/generated-sources/uniffi/swift/scmessenger_core.h iOS/SCMessenger/SCMessenger/Generated/apiFFI.h
cp core/target/generated-sources/uniffi/swift/api.modulemap iOS/SCMessenger/SCMessenger/Generated/apiFFI.modulemap || true

echo "Patching api.swift to remove nonisolated(unsafe) to fix Swift concurrency compilation errors..."
python3 -c "
import sys
path = 'iOS/SCMessenger/SCMessenger/Generated/api.swift'
with open(path, 'r') as f:
    code = f.read()

# Replace nonisolated(unsafe) with static func to avoid Swift 6 concurrency errors
code = code.replace('nonisolated(unsafe) static func', 'static func')
code = code.replace('public nonisolated(unsafe) static func', 'public static func')

with open(path, 'w') as f:
    f.write(code)
"

echo "Done!"
