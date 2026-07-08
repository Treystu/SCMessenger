# TASK: P1-06 follow-up - unit tests for mDNS self-loopback filter

Core fix already applied+committed:
android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt
`onServiceResolved` now filters any resolved peer-id matching `getLocalPeerId()`
before onPeerDiscovered/onLanPeerResolved fire. Wiring confirmed live via
TransportManager (`getLocalPeerId = getLocalPeerId` passthrough).

Deferred (2026-07-07, quota-constrained): the required unit tests.
- Test 1: resolved record peer-id == local identity -> onLanPeerResolved NOT invoked.
- Test 2: resolved record peer-id != local identity -> onLanPeerResolved still invoked (no regression).
`onServiceResolved` is invoked via NsdManager's private ResolveListener, so
testing needs either reflection (see AndroidPlatformBridgeTest.kt for an
existing reflection-invocation pattern in this codebase) or a refactor to
extract a testable pure function. Prefer reflection to avoid a production
signature change.
GATE: gradlew :app:testDebugUnitTest.
