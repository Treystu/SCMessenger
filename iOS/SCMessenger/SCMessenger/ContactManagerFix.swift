// Fixed ContactManager implementation to work around UniFFI generation issues
// This provides a proper class structure that conforms to ContactManagerProtocol

import Foundation

open class ContactManagerFixed: ContactManagerProtocol {
    private let generatedManager: ContactManager
    
    public init(storagePath: String) throws {
        self.generatedManager = try ContactManager(storagePath: storagePath)
    }
    
    // MARK: - ContactManagerProtocol Methods
    
    open func add(contact: Contact) throws {
        try generatedManager.add(contact: contact)
    }
    
    open func count() -> UInt32 {
        return generatedManager.count()
    }
    
    open func flush() {
        generatedManager.flush()
    }
    
    open func get(peerId: String) throws -> Contact? {
        return try generatedManager.get(peerId: peerId)
    }
    
    open func list() throws -> [Contact] {
        return try generatedManager.list()
    }
    
    open func remove(peerId: String) throws {
        try generatedManager.remove(peerId: peerId)
    }
    
    open func search(query: String) throws -> [Contact] {
        return try generatedManager.search(query: query)
    }
    
    open func setLocalNickname(peerId: String, nickname: String?) throws {
        try generatedManager.setLocalNickname(peerId: peerId, nickname: nickname)
    }
    
    open func setNickname(peerId: String, nickname: String?) throws {
        try generatedManager.setNickname(peerId: peerId, nickname: nickname)
    }
    
    open func updateDeviceId(peerId: String, deviceId: String?) throws {
        try generatedManager.updateDeviceId(peerId: peerId, deviceId: deviceId)
    }
    
    open func updateLastSeen(peerId: String) throws {
        try generatedManager.updateLastSeen(peerId: peerId)
    }
    
    // MARK: - FFI Methods (for internal use)
    
    func uniffiClonePointer() -> UnsafeMutableRawPointer {
        return generatedManager.uniffiClonePointer()
    }
}

// Extend the generated ContactManager to make it work properly
extension ContactManager: ContactManagerProtocol {
    // This extension makes the generated ContactManager conform to its protocol
    // by providing proper method implementations that call the generated FFI methods
    
    public func add(contact: Contact) throws {
        try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_add(self.uniffiClonePointer(),
                FfiConverterTypeContact.lower(contact), $0
            )
        }
    }
    
    public func count() -> UInt32 {
        return try! rustCall() {
            uniffi_scmessenger_core_fn_method_contactmanager_count(self.uniffiClonePointer(), $0)
        }
    }
    
    public func flush() {
        try! rustCall() {
            uniffi_scmessenger_core_fn_method_contactmanager_flush(self.uniffiClonePointer(), $0)
        }
    }
    
    public func get(peerId: String) throws -> Contact? {
        return try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_get(self.uniffiClonePointer(),
                FfiConverterString.lower(peerId), $0
            )
        }
    }
    
    public func list() throws -> [Contact] {
        return try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_list(self.uniffiClonePointer(), $0)
        }
    }
    
    public func remove(peerId: String) throws {
        try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_remove(self.uniffiClonePointer(),
                FfiConverterString.lower(peerId), $0
            )
        }
    }
    
    public func search(query: String) throws -> [Contact] {
        return try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_search(self.uniffiClonePointer(),
                FfiConverterString.lower(query), $0
            )
        }
    }
    
    public func setLocalNickname(peerId: String, nickname: String?) throws {
        try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_set_local_nickname(self.uniffiClonePointer(),
                FfiConverterString.lower(peerId),
                FfiConverterOptionString.lower(nickname), $0
            )
        }
    }
    
    public func setNickname(peerId: String, nickname: String?) throws {
        try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_set_nickname(self.uniffiClonePointer(),
                FfiConverterString.lower(peerId),
                FfiConverterOptionString.lower(nickname), $0
            )
        }
    }
    
    public func updateDeviceId(peerId: String, deviceId: String?) throws {
        try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_update_device_id(self.uniffiClonePointer(),
                FfiConverterString.lower(peerId),
                FfiConverterOptionString.lower(deviceId), $0
            )
        }
    }
    
    public func updateLastSeen(peerId: String) throws {
        try rustCallWithError(FfiConverterTypeIronCoreError.lift()) {
            uniffi_scmessenger_core_fn_method_contactmanager_update_last_seen(self.uniffiClonePointer(),
                FfiConverterString.lower(peerId), $0
            )
        }
    }
}

// Use the fixed implementation instead of the generated one
typealias ContactManager = ContactManagerFixed