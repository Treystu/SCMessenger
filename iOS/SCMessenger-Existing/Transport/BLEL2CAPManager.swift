//
//  BLEL2CAPManager.swift
//  SCMessenger
//
//  L2CAP channel management for bulk data transfer
//  Mirrors: android/.../transport/ble/BleL2capManager.kt
//

import CoreBluetooth
import os

/// L2CAP channel management for bulk data transfer
///
/// L2CAP provides connection-oriented channels over BLE for efficient bulk transfer
/// Used when GATT characteristics are too small/slow for large messages
final class BLEL2CAPManager: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-L2CAP")
    private weak var meshRepository: MeshRepository?
    
    // Channel tracking
    private var openChannels: [UUID: CBL2CAPChannel] = [:]
    private var publishedPSM: CBL2CAPPSM?
    
    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
    }
    
    // MARK: - Public API
    
    /// Open L2CAP channel to peripheral (central role)
    func openChannel(to peripheral: CBPeripheral, psm: CBL2CAPPSM) {
        logger.info("Opening L2CAP channel to \(peripheral.identifier), PSM: \(psm)")
        peripheral.openL2CAPChannel(psm)
    }
    
    /// Publish L2CAP PSM for incoming connections (peripheral role)
    func publishChannel(psm: CBL2CAPPSM, peripheralManager: CBPeripheralManager) {
        logger.info("Publishing L2CAP channel, PSM: \(psm)")
        peripheralManager.publishL2CAPChannel(withEncryption: true)
        publishedPSM = psm
    }
    
    /// Send data over L2CAP channel
    func sendData(_ data: Data, on channel: CBL2CAPChannel) {
        guard let outputStream = channel.outputStream else {
            logger.error("Output stream not available")
            return
        }
        
        guard outputStream.hasSpaceAvailable else {
            logger.warning("Output stream not ready")
            return
        }
        
        data.withUnsafeBytes { (bytes: UnsafeRawBufferPointer) in
            guard let baseAddress = bytes.baseAddress else { return }
            let buffer = baseAddress.assumingMemoryBound(to: UInt8.self)
            let bytesWritten = outputStream.write(buffer, maxLength: data.count)
            
            if bytesWritten < 0 {
                logger.error("Write error: \(outputStream.streamError?.localizedDescription ?? "unknown")")
            } else if bytesWritten < data.count {
                logger.warning("Partial write: \(bytesWritten)/\(data.count) bytes")
            } else {
                logger.debug("Sent \(bytesWritten) bytes over L2CAP")
            }
        }
    }
    
    /// Close L2CAP channel
    func closeChannel(_ channel: CBL2CAPChannel) {
        logger.info("Closing L2CAP channel")
        channel.inputStream.close()
        channel.outputStream.close()
        
        // Remove from tracking
        if let peer = openChannels.first(where: { $0.value === channel })?.key {
            openChannels.removeValue(forKey: peer)
        }
    }
    
    // MARK: - Internal Helpers
    
    func handleChannelOpened(_ channel: CBL2CAPChannel, for peripheral: CBPeripheral) {
        logger.info("L2CAP channel opened for \(peripheral.identifier)")
        openChannels[peripheral.identifier] = channel
        
        // Setup stream delegates for reading
        channel.inputStream.delegate = self
        channel.inputStream.schedule(in: .current, forMode: .default)
        channel.inputStream.open()
        
        channel.outputStream.delegate = self
        channel.outputStream.schedule(in: .current, forMode: .default)
        channel.outputStream.open()
    }
    
    func handleChannelClosed(_ channel: CBL2CAPChannel, error: Error?) {
        if let error = error {
            logger.error("L2CAP channel closed with error: \(error.localizedDescription)")
        } else {
            logger.info("L2CAP channel closed")
        }
        closeChannel(channel)
    }
}

// MARK: - StreamDelegate

extension BLEL2CAPManager: StreamDelegate {
    func stream(_ aStream: Stream, handle eventCode: Stream.Event) {
        switch eventCode {
        case .hasBytesAvailable:
            guard let inputStream = aStream as? InputStream else { return }
            readData(from: inputStream)
            
        case .hasSpaceAvailable:
            logger.debug("L2CAP stream has space available")
            
        case .errorOccurred:
            logger.error("L2CAP stream error: \(aStream.streamError?.localizedDescription ?? "unknown")")
            
        case .endEncountered:
            logger.info("L2CAP stream end encountered")
            
        default:
            break
        }
    }
    
    private func readData(from inputStream: InputStream) {
        let bufferSize = 1024
        var buffer = [UInt8](repeating: 0, count: bufferSize)
        
        let bytesRead = inputStream.read(&buffer, maxLength: bufferSize)
        
        if bytesRead > 0 {
            let data = Data(buffer.prefix(bytesRead))
            logger.debug("Received \(bytesRead) bytes over L2CAP")
            
            // Find peer ID for this stream
            if let channel = openChannels.first(where: { $0.value.inputStream === inputStream }),
               let peerId = channel.key.uuidString as String? {
                meshRepository?.onBleDataReceived(peerId: peerId, data: data)
            }
        } else if bytesRead < 0 {
            logger.error("Read error: \(inputStream.streamError?.localizedDescription ?? "unknown")")
        }
    }
}
