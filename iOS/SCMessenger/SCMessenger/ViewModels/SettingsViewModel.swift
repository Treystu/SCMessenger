//
//  SettingsViewModel.swift
//  SCMessenger
//
//  ViewModel for settings management
//  Mirrors: android/.../ui/viewmodels/SettingsViewModel.kt
//

import Foundation
import os

@MainActor
@Observable
final class SettingsViewModel {
    private weak var repository: MeshRepository?

    var settings: MeshSettings?
    var isLoading = false
    var isSaving = false
    var error: String?
    var successMessage: String?
    private var lastSettingUpdateTime: Date = .distantPast
    private let settingDebounceInterval: TimeInterval = 0.5  // 500ms

    init(repository: MeshRepository) {
        self.repository = repository
        self.loadNickname()
    }

    private func debouncedUpdateSettings(_ update: @escaping () -> Void) {
        let now = Date()
        if now.timeIntervalSince(lastSettingUpdateTime) < settingDebounceInterval {
            os_log("Settings update throttled")
            return
        }
        lastSettingUpdateTime = now
        update()
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

    // MARK: - Identity Backup (passphrase-encrypted)

    var backupExportResult: Result<String, Error>?
    var isExportingBackup = false

    /// Export a passphrase-encrypted identity backup (identity key + ratchet
    /// sessions + contacts). Distinct from `getIdentityExportString()`, which
    /// exports the public identity card with no encryption.
    ///
    /// Runs the actual Argon2id-backed encryption off the main actor: the
    /// repository call goes straight through to a plain (non-actor-isolated)
    /// UniFFI method that's safe to call from any thread, so doing it here
    /// synchronously would otherwise freeze the Settings UI for the KDF's
    /// duration (T16).
    func exportIdentityBackup(passphrase: String) {
        guard let core = repository?.ironCore else {
            backupExportResult = .failure(MeshError.notInitialized("IronCore not initialized"))
            return
        }
        isExportingBackup = true
        Task.detached(priority: .userInitiated) { [weak self] in
            let result: Result<String, Error>
            do {
                result = .success(try core.exportIdentityBackup(passphrase: passphrase))
            } catch {
                result = .failure(error)
            }
            await MainActor.run {
                self?.isExportingBackup = false
                self?.backupExportResult = result
            }
        }
    }

    func clearBackupExportResult() {
        backupExportResult = nil
    }

    var backupImportResult: Result<Void, Error>?
    var isImportingBackup = false

    /// Import an identity backup using a user-supplied passphrase. Same
    /// off-main-actor treatment as `exportIdentityBackup` (T16); mirrors
    /// `MeshRepository.importIdentityBackup`'s post-import consent grant.
    func importIdentityBackup(backup: String, passphrase: String) {
        guard let core = repository?.ironCore else {
            backupImportResult = .failure(MeshError.notInitialized("IronCore not initialized"))
            return
        }
        isImportingBackup = true
        Task.detached(priority: .userInitiated) { [weak self] in
            let result: Result<Void, Error>
            do {
                try core.importIdentityBackup(backup: backup, passphrase: passphrase)
                if core.getIdentityInfo().initialized {
                    core.grantConsent()
                }
                result = .success(())
            } catch {
                result = .failure(error)
            }
            await MainActor.run {
                self?.isImportingBackup = false
                switch result {
                case .success:
                    self?.successMessage = "Identity restored successfully"
                case .failure(let error):
                    self?.error = "Failed to restore identity: \(error.localizedDescription)"
                }
                self?.backupImportResult = result
            }
        }
    }

    func clearBackupImportResult() {
        backupImportResult = nil
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
        debouncedUpdateSettings { self.saveSettings() }
    }

    // MARK: - Transport Toggles (mirrors Android MeshSettingsScreen)

    func updateBleEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.bleEnabled = enabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    func updateWifiAwareEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.wifiAwareEnabled = enabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    func updateWifiDirectEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.wifiDirectEnabled = enabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    func updateInternetEnabled(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.internetEnabled = enabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    // MARK: - Discovery & Mesh Settings

    func updateDiscoveryMode(_ mode: DiscoveryMode) {
        guard var currentSettings = settings else { return }
        currentSettings.discoveryMode = mode
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }



    func updateBatteryFloor(_ floor: UInt8) {
        guard var currentSettings = settings else { return }
        currentSettings.batteryFloor = floor
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
    }

    // MARK: - Privacy / Onion Routing

    func updateOnionRouting(_ enabled: Bool) {
        guard var currentSettings = settings else { return }
        currentSettings.onionRouting = enabled
        settings = currentSettings
        debouncedUpdateSettings { self.saveSettings() }
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
        get { settings?.notificationsEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notificationsEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
            if newValue {
                Task {
                    _ = await NotificationManager.shared.requestPermissionIfNeeded()
                }
            }
        }
    }

    var notifyDmEnabled: Bool {
        get { settings?.notifyDmEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var notifyDmRequestEnabled: Bool {
        get { settings?.notifyDmRequestEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmRequestEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var notifyDmInForeground: Bool {
        get { settings?.notifyDmInForeground ?? false }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmInForeground = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var notifyDmRequestInForeground: Bool {
        get { settings?.notifyDmRequestInForeground ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.notifyDmRequestInForeground = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var soundEnabled: Bool {
        get { settings?.soundEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.soundEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var badgeEnabled: Bool {
        get { settings?.badgeEnabled ?? true }
        set {
            guard var currentSettings = settings else { return }
            currentSettings.badgeEnabled = newValue
            settings = currentSettings
            debouncedUpdateSettings { self.saveSettings() }
        }
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
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var isMessagePaddingEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_message_padding") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_message_padding")
            guard var s = settings else { return }
            s.messagePaddingEnabled = newValue
            settings = s
            debouncedUpdateSettings { self.saveSettings() }
        }
    }

    var isTimingObfuscationEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "privacy_timing_obfuscation") as? Bool ?? false }
        set {
            UserDefaults.standard.set(newValue, forKey: "privacy_timing_obfuscation")
            guard var s = settings else { return }
            s.timingObfuscationEnabled = newValue
            settings = s
            debouncedUpdateSettings { self.saveSettings() }
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

    // MARK: - Factory Reset (mirrors Android resetAllData)

    /// Wipe all local data and return the app to first-run state.
    func resetAllData() {
        Task { @MainActor in
            repository?.resetAllData()
        }
    }
}
