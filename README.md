# SCMessenger

**Status**: Active
**Last updated**: 2026-07-11
**Version**: v0.3.5 (alpha, driving to v1.0.0)

[![CI](https://github.com/Treystu/SCMessenger/actions/workflows/ci.yml/badge.svg)](https://github.com/Treystu/SCMessenger/actions/workflows/ci.yml)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**Messaging that works when the internet does not.**

SCMessenger is a sovereign, end-to-end encrypted, decentralized messaging
mesh. No servers. No accounts. No phone numbers. Your identity is a keypair
you generate on-device; your messages travel over whatever path physically
exists between you and your peer -- Bluetooth in a crowd, WiFi on a plane,
LAN at home, or a relay across the internet -- racing every available
transport and delivering over the first one that lands.

## Why it exists

Every mainstream messenger dies with its servers: censored, subpoenaed,
rate-limited, or simply offline. SCMessenger assumes the worst-case network
from the start -- no internet, no WiFi, a stranger's phone passing by on
BLE -- and treats the happy path as a bonus. If any radio on your device can
reach any radio on theirs, directly or through intermediate custody, the
message gets through.

## How it works

**Transport ladder, raced in parallel (sub-500ms failover):**
