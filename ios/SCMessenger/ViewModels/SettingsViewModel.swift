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
}
