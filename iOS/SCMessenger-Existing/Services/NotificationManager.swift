//
//  NotificationManager.swift
//  SCMessenger
//
//  Notification management with UNUserNotificationCenter
//

import UserNotifications
import os

/// Manages local notifications for incoming messages
final class NotificationManager: NSObject {
    static let shared = NotificationManager()
    
    private let logger = Logger(subsystem: "com.scmessenger", category: "Notifications")
    private let center = UNUserNotificationCenter.current()
    
    private override init() {
        super.init()
        center.delegate = self
    }
    
    // MARK: - Permission
    
    func requestPermission() async -> Bool {
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
    
    func sendMessageNotification(from sender: String, content: String, messageId: String) {
        let notificationContent = UNMutableNotificationContent()
        notificationContent.title = sender
        notificationContent.body = content
        notificationContent.sound = .default
        notificationContent.badge = NSNumber(value: getUnreadCount() + 1)
        notificationContent.categoryIdentifier = "MESSAGE"
        notificationContent.userInfo = ["messageId": messageId, "sender": sender]
        
        let request = UNNotificationRequest(
            identifier: messageId,
            content: notificationContent,
            trigger: nil // Deliver immediately
        )
        
        center.add(request) { [weak self] error in
            if let error = error {
                self?.logger.error("Failed to send notification: \(error.localizedDescription)")
            } else {
                self?.logger.debug("Notification sent for message \(messageId)")
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
        updateBadge(count: 0)
    }
    
    private func getUnreadCount() -> Int {
        // Would get actual unread count from repository
        return 0
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
            identifier: "MESSAGE",
            actions: [replyAction, markReadAction],
            intentIdentifiers: [],
            options: []
        )
        
        center.setNotificationCategories([messageCategory])
    }
}

// MARK: - UNUserNotificationCenterDelegate

extension NotificationManager: UNUserNotificationCenterDelegate {
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {
        // Show notification even when app is in foreground
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
            // User tapped the notification
            handleNotificationTap(userInfo: userInfo)
            
        default:
            break
        }
    }
    
    private func handleReply(text: String, userInfo: [AnyHashable: Any]) {
        guard let sender = userInfo["sender"] as? String else { return }
        logger.info("Quick reply to \(sender): \(text)")
        // Send message via repository
    }
    
    private func handleMarkAsRead(userInfo: [AnyHashable: Any]) {
        guard let messageId = userInfo["messageId"] as? String else { return }
        logger.info("Mark as read: \(messageId)")
        // Mark message as read in repository
    }
    
    private func handleNotificationTap(userInfo: [AnyHashable: Any]) {
        guard let sender = userInfo["sender"] as? String else { return }
        logger.info("Open chat with \(sender)")
        // Navigate to chat (via deep link or notification)
    }
}
