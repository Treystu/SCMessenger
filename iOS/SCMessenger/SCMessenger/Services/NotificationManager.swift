//
//  NotificationManager.swift
//  SCMessenger
//
//  Notification management with UNUserNotificationCenter
//

import Foundation
import UserNotifications
import os

extension Notification.Name {
    static let notificationRouteRequested = Notification.Name("com.scmessenger.notification-route-requested")
}

/// Manages local notifications for incoming messages
final class NotificationManager: NSObject {
    static let shared = NotificationManager()

    private let logger = Logger(subsystem: "com.scmessenger", category: "Notifications")
    private let center = UNUserNotificationCenter.current()
    private weak var repository: MeshRepository?
    private var notificationConversationMap: [String: String] = [:]
    private var unreadMessageIds = Set<String>()

    private enum Category {
        static let directMessage = "DIRECT_MESSAGE"
        static let directMessageRequest = "DIRECT_MESSAGE_REQUEST"
    }

    private enum UserInfoKey {
        static let messageId = "messageId"
        static let senderPeerId = "senderPeerId"
        static let senderDisplayName = "senderDisplayName"
        static let conversationId = "conversationId"
        static let routeTarget = "routeTarget"
    }

    private override init() {
        super.init()
        center.delegate = self
    }

    func configure(repository: MeshRepository) {
        self.repository = repository
        setupNotificationCategories()
    }

    // MARK: - Permission

    func requestPermissionIfNeeded() async -> Bool {
        let settings = await center.notificationSettings()
        switch settings.authorizationStatus {
        case .authorized, .provisional, .ephemeral:
            return true
        case .denied:
            logger.warning("Notification permission denied")
            return false
        case .notDetermined:
            return await requestPermission()
        @unknown default:
            return false
        }
    }

    private func requestPermission() async -> Bool {
        do {
            let granted = try await center.requestAuthorization(options: [.alert, .sound, .badge])
            logger.info("Notification permission: \(granted)")
            return granted
        } catch {
            logger.error("Failed to request notification permission: \(error.localizedDescription)")
            return false
        }
    }

    // MARK: - Message Notifications

    func sendNotification(
        decision: NotificationDecision,
        senderDisplayName: String,
        content: String,
        soundEnabled: Bool,
        badgeEnabled: Bool,
        routesToRequestsInbox: Bool
    ) {
        guard decision.shouldAlert else { return }

        let notificationContent = UNMutableNotificationContent()
        notificationContent.title = title(for: decision.kind, senderDisplayName: senderDisplayName)
        notificationContent.body = body(for: decision.kind, senderDisplayName: senderDisplayName, content: content)
        if soundEnabled {
            notificationContent.sound = .default
        }
        if badgeEnabled {
            unreadMessageIds.insert(decision.messageId)
            notificationContent.badge = NSNumber(value: unreadMessageIds.count)
        }
        notificationContent.categoryIdentifier = categoryIdentifier(for: decision.kind)
        notificationContent.userInfo = [
            UserInfoKey.messageId: decision.messageId,
            UserInfoKey.senderPeerId: decision.senderPeerId,
            UserInfoKey.senderDisplayName: senderDisplayName,
            UserInfoKey.conversationId: decision.conversationId,
            UserInfoKey.routeTarget: routesToRequestsInbox ? "requests" : "chat",
        ]
        notificationConversationMap[decision.messageId] = decision.conversationId

        let request = UNNotificationRequest(
            identifier: decision.messageId,
            content: notificationContent,
            trigger: nil
        )

        center.add(request) { [weak self] error in
            if let error = error {
                self?.logger.error("Failed to send notification: \(error.localizedDescription)")
            } else {
                self?.logger.debug("Notification sent for message \(decision.messageId)")
            }
        }
    }

    // MARK: - Badge Management

    func updateBadge(count: Int) {
        center.setBadgeCount(count) { [weak self] error in
            if let error = error {
                self?.logger.error("Failed to update badge: \(error.localizedDescription)")
            }
        }
    }

    func clearBadge() {
        unreadMessageIds.removeAll()
        updateBadge(count: 0)
    }

    func markMessageRead(messageId: String?) {
        guard let messageId else {
            clearBadge()
            return
        }
        unreadMessageIds.remove(messageId)
        notificationConversationMap.removeValue(forKey: messageId)
        center.removeDeliveredNotifications(withIdentifiers: [messageId])
        updateBadge(count: unreadMessageIds.count)
    }

    func markConversationRead(conversationId: String) {
        let matching = notificationConversationMap
            .filter { $0.value == conversationId }
            .map(\.key)
        for messageId in matching {
            unreadMessageIds.remove(messageId)
            notificationConversationMap.removeValue(forKey: messageId)
        }
        if !matching.isEmpty {
            center.removeDeliveredNotifications(withIdentifiers: matching)
        }
        updateBadge(count: unreadMessageIds.count)
    }

    private func title(for kind: NotificationKind, senderDisplayName: String) -> String {
        switch kind {
        case .directMessage:
            return senderDisplayName
        case .directMessageRequest:
            return "Message Request"
        case .none:
            return senderDisplayName
        }
    }

    private func body(for kind: NotificationKind, senderDisplayName: String, content: String) -> String {
        switch kind {
        case .directMessage:
            return content
        case .directMessageRequest:
            return "\(senderDisplayName): \(content)"
        case .none:
            return content
        }
    }

    private func categoryIdentifier(for kind: NotificationKind) -> String {
        switch kind {
        case .directMessage:
            return Category.directMessage
        case .directMessageRequest:
            return Category.directMessageRequest
        case .none:
            return Category.directMessage
        }
    }

    // MARK: - Notification Actions

    func setupNotificationCategories() {
        let replyAction = UNTextInputNotificationAction(
            identifier: "REPLY_ACTION",
            title: "Reply",
            options: [],
            textInputButtonTitle: "Send",
            textInputPlaceholder: "Message"
        )

        let markReadAction = UNNotificationAction(
            identifier: "MARK_READ_ACTION",
            title: "Mark as Read",
            options: []
        )

        let messageCategory = UNNotificationCategory(
            identifier: Category.directMessage,
            actions: [replyAction, markReadAction],
            intentIdentifiers: [],
            options: []
        )

        let requestCategory = UNNotificationCategory(
            identifier: Category.directMessageRequest,
            actions: [markReadAction],
            intentIdentifiers: [],
            options: []
        )

        center.setNotificationCategories([messageCategory, requestCategory])
    }
}

// MARK: - UNUserNotificationCenterDelegate

extension NotificationManager: UNUserNotificationCenterDelegate {
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {
        [.banner, .sound, .badge]
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse
    ) async {
        let userInfo = response.notification.request.content.userInfo

        switch response.actionIdentifier {
        case "REPLY_ACTION":
            if let textResponse = response as? UNTextInputNotificationResponse {
                handleReply(text: textResponse.userText, userInfo: userInfo)
            }

        case "MARK_READ_ACTION":
            handleMarkAsRead(userInfo: userInfo)

        case UNNotificationDefaultActionIdentifier:
            handleNotificationTap(userInfo: userInfo)

        default:
            break
        }
    }

    private func handleReply(text: String, userInfo: [AnyHashable: Any]) {
        guard let peerId = userInfo[UserInfoKey.senderPeerId] as? String else { return }
        let messageId = userInfo[UserInfoKey.messageId] as? String
        logger.info("Quick reply to \(peerId): \(text)")
        Task { @MainActor [weak self] in
            do {
                try await self?.repository?.sendMessage(peerId: peerId, content: text)
                self?.markMessageRead(messageId: messageId)
            } catch {
                self?.logger.error("Quick reply failed: \(error.localizedDescription)")
            }
        }
    }

    private func handleMarkAsRead(userInfo: [AnyHashable: Any]) {
        let messageId = userInfo[UserInfoKey.messageId] as? String
        if let messageId {
            logger.info("Mark as read: \(messageId)")
        }
        markMessageRead(messageId: messageId)
    }

    private func handleNotificationTap(userInfo: [AnyHashable: Any]) {
        let messageId = userInfo[UserInfoKey.messageId] as? String
        if let messageId {
            markMessageRead(messageId: messageId)
        }
        NotificationCenter.default.post(
            name: .notificationRouteRequested,
            object: nil,
            userInfo: userInfo
        )
        if let peerId = userInfo[UserInfoKey.senderPeerId] as? String {
            logger.info("Open notification route for \(peerId)")
        }
    }

    // MARK: - Verification Methods

    /// Verifies notification permission flow with comprehensive status reporting
    func verifyPermissionFlow() async -> PermissionVerificationResult {
        let settings = await center.notificationSettings()

        switch settings.authorizationStatus {
        case .notDetermined:
            logger.info("Permission: not determined, requesting...")
            let granted = await requestPermission()
            return PermissionVerificationResult(
                status: granted ? .granted : .denied,
                requiredAction: .requested,
                success: granted
            )

        case .denied:
            logger.warning("Permission: denied - user needs to enable in Settings")
            return PermissionVerificationResult(
                status: .denied,
                requiredAction: .manualEnable,
                success: false
            )

        case .authorized, .provisional, .ephemeral:
            logger.info("Permission: authorized")
            return PermissionVerificationResult(
                status: .authorized,
                requiredAction: .none,
                success: true
            )

        @unknown default:
            logger.error("Permission: unknown status")
            return PermissionVerificationResult(
                status: .unknown,
                requiredAction: .none,
                success: false
            )
        }
    }

    /// Returns current notification permission status as a string
    func currentPermissionStatus() -> String {
        let settings = center.notificationSettings()
        switch settings.authorizationStatus {
        case .notDetermined:
            return "not_determined"
        case .denied:
            return "denied"
        case .authorized:
            return "authorized"
        case .provisional:
            return "provisional"
        case .ephemeral:
            return "ephemeral"
        @unknown default:
            return "unknown"
        }
    }

    /// Returns whether notifications are currently enabled
    func areNotificationsEnabled() -> Bool {
        let settings = center.notificationSettings()
        return settings.authorizationStatus == .authorized ||
               settings.authorizationStatus == .provisional ||
               settings.authorizationStatus == .ephemeral
    }
}

/// Result of permission verification
struct PermissionVerificationResult {
    enum PermissionStatus {
        case authorized, denied, notDetermined, unknown
    }

    enum RequiredAction {
        case none, requested, manualEnable
    }

    let status: PermissionStatus
    let requiredAction: RequiredAction
    let success: Bool
}
