# Android Identity Mismatch Root Cause Analysis
**Date:** March 10, 2026
**Status:** INVESTIGATING

## Problem Statement

User reports:
1. Fresh install has pre-loaded identity (should be generating new one)
2. ID mismatches requiring unification
3. Android delivery still buggy after case-sensitivity fixes

## Investigation Plan

### 1. Identity Generation
- [ ] Verify Android generates fresh identity on first launch
- [ ] Check for identity persistence/caching issues
- [ ] Confirm identity matches across components

### 2. Peer ID Normalization
- [ ] Audit all peer ID comparisons (case-sensitivity)
- [ ] Check for mixed case/lowercase usage
- [ ] Verify canonical peer ID handling

### 3. Delivery State Tracking
- [ ] Trace message ID propagation
- [ ] Fix `msg=unknown` issue
- [ ] Verify UI updates on state changes

### 4. Cross-Platform Testing
- [ ] Android (cellular) ↔ iOS Sim (WiFi)
- [ ] Verify peer discovery
- [ ] Test message delivery end-to-end
- [ ] Confirm delivery status updates

## Initial Findings

Testing fresh Android install...

