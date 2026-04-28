import Foundation

@inline(__always)
private func expect(_ condition: @autoclosure () -> Bool, _ message: String) {
    if !condition() {
        fputs("FAIL: \(message)\n", stderr)
        exit(1)
    }
}

private func testMultipeerPathSuccess() {
    var attempted: [String] = []

    let result = LocalTransportFallback.attemptMultipeerThenBle(
        multipeerPeerId: "deadbeef",
        blePeerId: "f4d0b0b4-512f-465f-a5e4-39ee7bf7b1cb",
        tryMultipeer: { _ in
            attempted.append("multipeer")
            return true
        },
        tryBle: { _ in
            attempted.append("ble")
            return true
        }
    )

    expect(result.multipeerAttempted, "multipeer should be attempted when available")
    expect(result.multipeerAcked, "multipeer should ACK when send succeeds")
    expect(!result.bleAttempted, "BLE should not be attempted when multipeer succeeds")
    expect(!result.bleAcked, "BLE should remain unacked when not attempted")
    expect(result.acked, "overall result must be acked")
    expect(attempted == ["multipeer"], "fallback ordering should stop after multipeer success")
}

private func testFallbackWhenMultipeerUnavailable() {
    var attempted: [String] = []

    let result = LocalTransportFallback.attemptMultipeerThenBle(
        multipeerPeerId: "facefeed",
        blePeerId: "2e8e2a58-40e2-4c49-b0c7-8f3706e2ad90",
        tryMultipeer: { _ in
            attempted.append("multipeer")
            return false
        },
        tryBle: { _ in
            attempted.append("ble")
            return true
        }
    )

    expect(result.multipeerAttempted, "multipeer should be attempted first")
    expect(!result.multipeerAcked, "multipeer should not ACK when unavailable")
    expect(result.bleAttempted, "BLE must be attempted after multipeer failure")
    expect(result.bleAcked, "BLE should ACK when fallback succeeds")
    expect(result.acked, "overall result must still be acked")
    expect(attempted == ["multipeer", "ble"], "fallback ordering must be deterministic")
}

private func testReconnectContinuationAndThroughputStability() {
    var multipeerOnline = false

    // Reconnect continuation: first fallback to BLE, then resume Multipeer once online.
    let beforeReconnect = LocalTransportFallback.attemptMultipeerThenBle(
        multipeerPeerId: "cafebabe",
        blePeerId: "a8cd0a14-00a2-4b53-af4c-a12de228ec7c",
        tryMultipeer: { _ in multipeerOnline },
        tryBle: { _ in true }
    )
    expect(beforeReconnect.bleAttempted && beforeReconnect.bleAcked, "BLE fallback should carry delivery while multipeer is offline")

    multipeerOnline = true
    let afterReconnect = LocalTransportFallback.attemptMultipeerThenBle(
        multipeerPeerId: "cafebabe",
        blePeerId: "a8cd0a14-00a2-4b53-af4c-a12de228ec7c",
        tryMultipeer: { _ in multipeerOnline },
        tryBle: { _ in true }
    )
    expect(afterReconnect.multipeerAcked, "multipeer should resume once reconnected")
    expect(!afterReconnect.bleAttempted, "BLE should not be used after multipeer reconnect")

    // Throughput stability: deterministic behavior under sustained mixed availability.
    var multipeerCalls = 0
    var bleCalls = 0
    var multipeerSuccesses = 0
    var bleFallbackSuccesses = 0

    for index in 0..<1500 {
        let isMultipeerAvailable = index % 3 != 0
        let result = LocalTransportFallback.attemptMultipeerThenBle(
            multipeerPeerId: "a11ce123",
            blePeerId: "90bb12c3-f84f-48dc-a306-3d77df6964ab",
            tryMultipeer: { _ in
                multipeerCalls += 1
                return isMultipeerAvailable
            },
            tryBle: { _ in
                bleCalls += 1
                return true
            }
        )

        if isMultipeerAvailable {
            multipeerSuccesses += 1
            expect(result.multipeerAcked, "multipeer should ACK when available")
            expect(!result.bleAttempted, "BLE should be skipped on multipeer ACK")
        } else {
            bleFallbackSuccesses += 1
            expect(!result.multipeerAcked, "multipeer should not ACK when unavailable")
            expect(result.bleAttempted, "BLE should be attempted when multipeer is unavailable")
            expect(result.bleAcked, "BLE fallback should ACK")
        }

        expect(result.acked, "every iteration must remain loss-safe")
    }

    expect(multipeerCalls == 1500, "multipeer should be evaluated on every attempt")
    expect(bleCalls == 500, "BLE should be invoked only when multipeer is unavailable")
    expect(multipeerSuccesses == 1000, "multipeer success count should match availability pattern")
    expect(bleFallbackSuccesses == 500, "BLE fallback count should match availability pattern")
}

private func testBleOnlyTerminalFailureSignal() {
    var multipeerCalled = false
    var bleCalled = false

    let result = LocalTransportFallback.attemptMultipeerThenBle(
        multipeerPeerId: nil,
        blePeerId: "2cc7db8f-3142-4b8c-9157-2be2d7502e7f",
        tryMultipeer: { _ in
            multipeerCalled = true
            return true
        },
        tryBle: { _ in
            bleCalled = true
            return false
        }
    )

    expect(!multipeerCalled, "multipeer must not be called in BLE-only fallback path")
    expect(bleCalled, "BLE path should be attempted")
    expect(result.bleAttempted, "BLE should be marked attempted")
    expect(!result.bleAcked, "BLE should be marked failed")
    expect(!result.acked, "overall result must report deterministic terminal failure")
}

func runAllTests() {
    testMultipeerPathSuccess()
    testFallbackWhenMultipeerUnavailable()
    testReconnectContinuationAndThroughputStability()
    testBleOnlyTerminalFailureSignal()
    print("PASS: local transport fallback tests")
}

@main
struct LocalTransportFallbackTestRunner {
    static func main() {
        runAllTests()
    }
}
