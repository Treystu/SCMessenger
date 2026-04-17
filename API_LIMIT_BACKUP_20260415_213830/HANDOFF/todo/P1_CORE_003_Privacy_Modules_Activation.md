# P1_CORE_003: Privacy Modules Activation

**Priority:** P1 (Core Functionality)
**Platform:** Core/Rust
**Status:** Dormant (Not Wired)
**Source:** PRODUCTION_ROADMAP.md - Module Status Matrix

## Problem Description
Privacy modules (onion routing, cover traffic, padding, timing) are fully implemented but completely dormant - never called in production. This includes `privacy/onion.rs`, cover traffic, and timing obfuscation.

## Impact
- Missing privacy-enhancing features
- No traffic analysis resistance
- Reduced anonymity properties
- Wasted privacy investment

## Implementation Required
1. Wire privacy modules into message preparation
2. Integrate onion routing with transport layer
3. Activate cover traffic generation
4. Connect timing obfuscation to delivery

## Key Files
- `core/src/privacy/mod.rs` - Main module wiring
- `core/src/message/prepare.rs` - Integration
- `core/src/transport/manager.rs` - Transport integration

## Expected Outcome
- Onion routing capabilities active
- Cover traffic generation enabled
- Timing obfuscation operational
- Enhanced privacy protections