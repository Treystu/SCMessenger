//
//  SettingsViewModel.swift
//  SCMessenger
//
//  ViewModel for settings management
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
        self.nickname = name
        do {
            try repository?.setNickname(name)
            successMessage = "Nickname updated"
        } catch {
            self.error = "Failed to update nickname: \(error.localizedDescription)"
        }
    }

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
    
    func toggleRelay() {
        guard var currentSettings = settings else { return }
        currentSettings.relayEnabled = !currentSettings.relayEnabled
        settings = currentSettings
        saveSettings()
    }
    
    func updateDiscoveryMode(_ mode: DiscoveryMode) {
        guard var currentSettings = settings else { return }
        currentSettings.discoveryMode = mode
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
}
