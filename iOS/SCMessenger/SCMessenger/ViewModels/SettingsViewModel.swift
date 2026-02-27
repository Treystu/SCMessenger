//
//  SettingsViewModel.swift
//  SCMessenger
//
//  ViewModel for settings management
//  Mirrors: android/.../ui/viewmodels/SettingsViewModel.kt
//

import Foundation

@Observable
final class SettingsViewModel {
    private weak var repository: MeshRepository?

    var settings: MeshSettings?
    var isLoading = false
    var isSaving = false
    var error: String?
    var successMessage: String?

    init(repository: MeshRepository) {
        self.repository = repository
        self.loadNickname()
    }

    // MARK: - Identity

    var nickname: String = ""

    func loadNickname() {
        if let name = repository?.getNickname() {
            self.nickname = name
        } else {
            self.nickname = "User" // Default
        }
    }

    func updateNickname(_ name: String) {
        do {
            try repository?.setNickname(name)
            self.nickname = name
            successMessage = "Nickname updated"
        } catch {
            self.error = "Failed to update nickname: \(error.localizedDescription)"
        }
    }

    func getIdentityExportString() -> String {
        return repository?.getIdentityExportString() ?? "{}"
    }

    // MARK: - Settings Lifecycle

    func loadSettings() {
        isLoading = true
        do {
            settings = try repository?.loadSettings()
            error = nil
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func saveSettings() {
        guard let settings = settings else { return }

        isSaving = true
        do {
            try repository?.saveSettings(settings)
            successMessage = "Settings saved"
            error = nil
        } catch {
            self.error = error.localizedDescription
        }
        isSaving = false
    }

    // MARK: - Relay

    func toggleRelay() {
        guard var currentSettings = settings else { return }
        currentSettings.relayEnabled = !currentSettings.relayEnabled
        settings = currentSettings
        saveSettings()
    }

    // MARK: - Transport Toggles (mirrors Android MeshSettingsScreen)

    func updateBleEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.bleEnabled = enabled
        settings = currentSettings
        saveSettings()
    }

    func updateWifiAwareEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.wifiAwareEnabled = enabled
        settings = currentSettings
        saveSettings()
    }

    func updateWifiDirectEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.wifiDirectEnabled = enabled
        settings = currentSettings
        saveSettings()
    }

    func updateInternetEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.internetEnabled = enabled
        settings = currentSettings
        saveSettings()
    }

    // MARK: - Discovery & Mesh Settings

    func updateDiscoveryMode(_ mode: DiscoveryMode) {
        guard var currentSettings = settings else { return }
        currentSettings.discoveryMode = mode
        settings = currentSettings
        saveSettings()
    }



    func updateBatteryFloor(_ floor: UInt8) {
        guard var currentSettings = settings else { return }
        currentSettings.batteryFloor = floor
        settings = currentSettings
        saveSettings()
    }

    // MARK: - Privacy / Onion Routing

    func updateOnionRouting(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.onionRouting = enabled
        settings = currentSettings
        saveSettings()
    }

    // MARK: - BLE Privacy

    var isBleRotationEnabled: Bool {
        return repository?.blePrivacyEnabled ?? true
    }

    var bleRotationInterval: TimeInterval {
        return repository?.blePrivacyInterval ?? 900
    }

    func toggleBleRotation(enabled: Bool) {
        repository?.blePrivacyEnabled = enabled
    }

    func updateBleRotationInterval(_ interval: TimeInterval) {
        repository?.blePrivacyInterval = interval
    }

    // MARK: - App Preferences (mirrors Android PreferencesRepository)

    var isNotificationsEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "notifications_enabled") as? Bool ?? true }
        set { UserDefaults.standard.set(newValue, forKey: "notifications_enabled") }
    }

    // MARK: - Privacy Feature Preferences
    //
    // These preferences persist via UserDefaults and are forwarded to the Rust
    // core privacy module (cover traffic, padding, timing obfuscation) via
    // MeshSettings once the UniFFI surface exposes per-feature toggles.
    // Until then the stored value is the source of truth consulted by the UI.

    var isCoverTrafficEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_cover_traffic") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_cover_traffic")
            guard var s = settings else { return }
            s.coverTrafficEnabled = newValue
            settings = s
            saveSettings()
        }
    }

    var isMessagePaddingEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_message_padding") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_message_padding")
            guard var s = settings else { return }
            s.messagePaddingEnabled = newValue
            settings = s
            saveSettings()
        }
    }

    var isTimingObfuscationEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_timing_obfuscation") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_timing_obfuscation")
            guard var s = settings else { return }
            s.timingObfuscationEnabled = newValue
            settings = s
            saveSettings()
        }
    }

    // MARK: - Service Control

    var isServiceRunning: Bool {
        return repository?.serviceState == .running
    }

    func startService() {
        repository?.start()
    }

    func stopService() {
        repository?.stopMeshService()
    }

    // MARK: - Info Counts

    func getContactCount() -> UInt32 {
        return (try? repository?.getContactCount()) ?? 0
    }

    func getMessageCount() -> UInt32 {
        return (try? repository?.getMessageCount()) ?? 0
    }

    func getConnectionPathState() -> ConnectionPathState {
        return repository?.getConnectionPathState() ?? .disconnected
    }

    func getNatStatus() -> String {
        return repository?.getNatStatus() ?? "unknown"
    }

    func exportDiagnostics() -> String {
        return repository?.exportDiagnostics() ?? "{}"
    }

    // MARK: - Auto-Adjust (mirrors Android PowerSettingsScreen)

    var isAutoAdjustEnabled: Bool {
        get { repository?.isAutoAdjustEnabled ?? (UserDefaults.standard.object(forKey: "auto_adjust_enabled") as? Bool ?? true) }
        set { repository?.setAutoAdjustEnabled(newValue) }
    }


}
