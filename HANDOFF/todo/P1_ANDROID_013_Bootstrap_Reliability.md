# P1_ANDROID_013_Bootstrap_Reliability

**Priority:** P1
**Type:** IMPLEMENTATION
**Platform:** Android (Network)
**Estimated LoC Impact:** 50–100 LoC

## Objective
Address the bootstrap node connectivity failures that trigger the fallback protocol loop.

## Background
The logcat from P0_ANDROID_010 shows ALL bootstrap nodes failing with `NetworkException: Network error`:
- `/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw`
- `/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9`
- `/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw`

Every bootstrap attempt fails within milliseconds, which then triggers the fallback protocol, which also fails. This creates a rapid failure loop that contributes to CPU saturation and ANR.

## Requirements
1. **Exponential backoff for bootstrap failures** — If all bootstrap nodes fail, don't immediately retry. Implement exponential backoff (1s, 2s, 4s, 8s, max 60s) before the next bootstrap attempt.
2. **Bootstrap failure rate limiting** — If bootstrap has failed N consecutive times, pause bootstrap attempts for a cooldown period (e.g., 30 seconds).
3. **Better error classification** — Distinguish between "no network" (device offline) vs "server unreachable" (network up but bootstrap down) vs "timeout" (network up but slow). Only trigger fallback on server-unreachable, not on no-network.

## Verification Checklist
- [ ] Bootstrap failures are rate-limited with backoff
- [ ] No rapid failure loop in logcat when all nodes are down
- [ ] Normal bootstrap still works when nodes are available