//
//  Models.swift
//  SCMessenger
//
//  Shared model types for the iOS app
//

import Foundation

/// Conversation model for chat list
struct Conversation: Identifiable, Hashable {
    let id: String
    let peerId: String
    let peerNickname: String
    let lastMessage: String?
    let lastMessageTime: Date?
    let unreadCount: Int
    
    init(peerId: String, peerNickname: String, lastMessage: String? = nil, lastMessageTime: Date? = nil, unreadCount: Int = 0) {
        self.id = peerId
        self.peerId = peerId
        self.peerNickname = peerNickname
        self.lastMessage = lastMessage
        self.lastMessageTime = lastMessageTime
        self.unreadCount = unreadCount
    }
}

/// Transport type enumeration
enum TransportType: String, CaseIterable {
    case multipeer = "Multipeer"
    case ble = "BLE"
    case internet = "Internet"
    
    var icon: String {
        switch self {
        case .multipeer: return "wifi"
        case .ble: return "antenna.radiowaves.left.and.right"
        case .internet: return "network"
        }
    }
}

/// App-level settings (separate from MeshSettings)
struct AppSettings {
    var hasCompletedOnboarding: Bool = false
    var notificationsEnabled: Bool = true
    var appearance: AppAppearance = .auto
    
    enum AppAppearance: String {
        case auto = "Auto"
        case light = "Light"
        case dark = "Dark"
    }
}
