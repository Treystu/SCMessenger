//
//  MeshBackgroundServiceTests.swift
//  SCMessengerTests
//
//  Verifies the BGAppRefreshTask/BGProcessingTask scheduling contract:
//  the task identifiers registered with BGTaskScheduler must match the ones
//  declared in Info.plist's BGTaskSchedulerPermittedIdentifiers (a mismatch
//  makes BGTaskScheduler.register(...) assert/crash at launch), and the
//  handlers must actually drive the mesh sync work they're scheduled for.
//

import XCTest
@testable import SCMessenger

final class MeshBackgroundServiceTests: XCTestCase {
    private var meshRepository: MeshRepository!
    private var backgroundService: MeshBackgroundService!

    override func setUp() {
        super.setUp()
        meshRepository = MeshRepository()
        backgroundService = MeshBackgroundService(meshRepository: meshRepository)
    }

    override func tearDown() {
        backgroundService = nil
        meshRepository = nil
        super.tearDown()
    }

    /// The two identifiers MeshBackgroundService registers with
    /// BGTaskScheduler must be declared in Info.plist's
    /// BGTaskSchedulerPermittedIdentifiers, or registration asserts/crashes
    /// at launch on a real device.
    func testTaskIdentifiersAreDeclaredInInfoPlist() {
        guard let permittedIdentifiers = Bundle.main.object(
            forInfoDictionaryKey: "BGTaskSchedulerPermittedIdentifiers"
        ) as? [String] else {
            XCTFail("BGTaskSchedulerPermittedIdentifiers missing from Info.plist")
            return
        }

        XCTAssertTrue(
            permittedIdentifiers.contains(MeshBackgroundService.refreshTaskId),
            "refreshTaskId (\(MeshBackgroundService.refreshTaskId)) must be in BGTaskSchedulerPermittedIdentifiers"
        )
        XCTAssertTrue(
            permittedIdentifiers.contains(MeshBackgroundService.processingTaskId),
            "processingTaskId (\(MeshBackgroundService.processingTaskId)) must be in BGTaskSchedulerPermittedIdentifiers"
        )
    }

    /// Exercises the same work the real BGAppRefreshTask handler performs
    /// (sync pending messages + refresh stats) via the debug-only simulation
    /// hook, since the real BGTaskScheduler launch path can't be driven
    /// directly from a unit test.
    func testSimulatedBackgroundRefreshCompletesWithoutThrowing() async {
        let task = backgroundService.simulateBackgroundRefresh()
        await task.value
    }

    /// Exercises the same work the real BGProcessingTask handler performs
    /// (bulk sync + peer ledger update) via the debug-only simulation hook.
    func testSimulatedBackgroundProcessingCompletesWithoutThrowing() async {
        let task = backgroundService.simulateBackgroundProcessing()
        await task.value
    }
}
