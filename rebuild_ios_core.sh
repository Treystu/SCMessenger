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
cp core/target/generated-sources/uniffi/swift/api.swift iOS/SCMessenger/SCMessenger/Generated/api.swift
cp core/target/generated-sources/uniffi/swift/apiFFI.h iOS/SCMessenger/SCMessenger/Generated/apiFFI.h
cp core/target/generated-sources/uniffi/swift/apiFFI.modulemap iOS/SCMessenger/SCMessenger/Generated/apiFFI.modulemap || true

echo "Done!"
