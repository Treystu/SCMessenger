//
//  ReceiptUnificationTests.swift
//  SCMessengerTests
//
//  Tests unified receipt encoding/decoding via core's FFI functions.
//  Mirrors: A-04 Android ReceiptUnificationTest.kt pattern
//
//  These tests verify that receipt encoding/decoding uses the unified core functions
//  across all platforms, ensuring consistent wire format and preventing platform-specific bugs.
//

import XCTest
@testable import SCMessenger

final class ReceiptUnificationTests: XCTestCase {

    /// Test round-trip receipt encoding and decoding using unified core functions
    /// - Encodes receipt components via core's encodeReceiptFromComponents()
    /// - Parses it back via core's decodeReceiptToComponents()
    /// - Verifies no data loss in round-trip
    func testRoundTripReceiptEncoding() throws {
        let messageId: String = "test-message-123"
        let status: String = "Delivered"
        let timestamp: UInt64 = 1234567890

        // Encode via core function (unified across all platforms)
        let encodedData: Data = try encodeReceiptFromComponents(
            messageId: messageId,
            status: status,
            timestamp: timestamp
        )

        // Verify encoded data is not empty
        XCTAssertGreaterThan(
            encodedData.count,
            0,
            "Encoded receipt should not be empty"
        )

        // Decode back via core function (unified across all platforms)
        let decodedReceipt: ReceiptComponents = try decodeReceiptToComponents(data: encodedData)

        // Verify round-trip integrity
        XCTAssertEqual(
            decodedReceipt.message_id,
            messageId,
            "Decoded message_id should match original"
        )
        XCTAssertEqual(
            decodedReceipt.status,
            status,
            "Decoded status should match original"
        )
        XCTAssertEqual(
            decodedReceipt.timestamp,
            timestamp,
            "Decoded timestamp should match original"
        )
    }

    /// Test encoding different delivery statuses
    func testEncodeReceiptWithDifferentStatuses() throws {
        let messageId: String = "msg-456"
        let timestamp: UInt64 = 9876543210

        for status in ["Sent", "Delivered"] {
            // Encode components
            let encodedData: Data = try encodeReceiptFromComponents(
                messageId: messageId,
                status: status,
                timestamp: timestamp
            )

            // Decode and verify
            let decodedReceipt: ReceiptComponents = try decodeReceiptToComponents(data: encodedData)

            XCTAssertEqual(
                decodedReceipt.status,
                status,
                "Status '\(status)' should round-trip correctly"
            )
        }
    }

    /// Test decoding invalid data raises error
    func testDecodingInvalidDataThrows() throws {
        let invalidData: Data = Data([0xFF, 0xFE, 0xFD])

        XCTAssertThrowsError(
            try decodeReceiptToComponents(data: invalidData),
            "Decoding invalid data should throw"
        )
    }
}
