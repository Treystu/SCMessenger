//
//  SettingsView.swift
//  SCMessenger
//
//  Main settings view
//

import SwiftUI

struct SettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: SettingsViewModel?
    
    var body: some View {
        Form {
            Section {
                RelayToggleRow(
                    isEnabled: Binding(
                        get: { viewModel?.settings?.relayEnabled ?? false },
                        set: { _ in viewModel?.toggleRelay() }
                    )
                )
                
                RelayWarningCard()
            } header: {
                Text("Relay & Messaging")
            }
            
            Section {
                NavigationLink("Mesh Settings") {
                    MeshSettingsView()
                }
                
                NavigationLink("Privacy Settings") {
                    PrivacySettingsView()
                }
            } header: {
                Text("Advanced")
            }
            
            Section {
                HStack {
                    Text("Version")
                    Spacer()
                    Text("0.1.1")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
                
                HStack {
                    Text("Identity")
                    Spacer()
                    Text(repository.getIdentitySnippet())
                        .font(.system(.body, design: .monospaced))
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            } header: {
                Text("About")
            }
        }
        .navigationTitle("Settings")
        .onAppear {
            if viewModel == nil {
                viewModel = SettingsViewModel(repository: repository)
                viewModel?.loadSettings()
            }
        }
    }
}

struct RelayToggleRow: View {
    @Binding var isEnabled: Bool
    
    var body: some View {
        Toggle(isOn: $isEnabled) {
            HStack(spacing: Theme.spacingSmall) {
                Image(systemName: "antenna.radiowaves.left.and.right")
                    .foregroundStyle(Theme.onErrorContainer)
                Text("Enable Relay")
                    .font(Theme.titleMedium.weight(.medium))
            }
        }
        .tint(Theme.onErrorContainer)
    }
}

struct RelayWarningCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingSmall) {
            HStack(spacing: Theme.spacingSmall) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(Theme.onErrorContainer)
                Text("Critical: Relay = Messaging")
                    .font(Theme.titleSmall.weight(.medium))
                    .foregroundStyle(Theme.onErrorContainer)
            }
            
            VStack(alignment: .leading, spacing: Theme.spacingXSmall) {
                BulletPoint("Relay enabled: You can send and receive messages")
                BulletPoint("Relay disabled: All messaging blocked")
                BulletPoint("You relay for others, they relay for you")
            }
            .font(Theme.bodySmall.weight(.medium))
        }
        .padding(Theme.spacingMedium)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Theme.errorContainer)
        .cornerRadius(Theme.cornerRadiusSmall)
    }
}

struct MeshSettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: SettingsViewModel?
    
    private var discoveryModeBinding: Binding<DiscoveryMode> {
        return Binding(
            get: { viewModel?.settings?.discoveryMode ?? .normal },
            set: { viewModel?.updateDiscoveryMode($0) }
        )
    }

    var body: some View {
        Form {
            Section("Discovery") {
                Picker("Mode", selection: discoveryModeBinding) {
                    Text("Aggressive (Normal)").tag(DiscoveryMode.normal)
                    Text("Balanced (Cautious)").tag(DiscoveryMode.cautious)
                    Text("Passive (Paranoid)").tag(DiscoveryMode.paranoid)
                }
            }
            
            Section("Battery") {
                HStack {
                    Text("Minimum Level")
                    Spacer()
                    Text("\(viewModel?.settings?.batteryFloor ?? 20)%")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            }
        }
        .navigationTitle("Mesh Settings")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            if viewModel == nil {
                viewModel = SettingsViewModel(repository: repository)
                viewModel?.loadSettings()
            }
        }
    }
}

struct PrivacySettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: SettingsViewModel?
    @State private var isRotationEnabled = true
    
    var body: some View {
        Form {
            Section("Privacy") {
                Toggle("Rotate BLE Identity", isOn: Binding(
                    get: { viewModel?.isBleRotationEnabled ?? true },
                    set: { val in
                        viewModel?.toggleBleRotation(enabled: val)
                        isRotationEnabled = val // Force refresh
                    }
                ))
                
                HStack {
                    Text("Rotation Interval")
                    Spacer()
                    Text("\(Int((viewModel?.bleRotationInterval ?? 900) / 60)) min")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            }
        }
        .navigationTitle("Privacy Settings")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            if viewModel == nil {
                viewModel = SettingsViewModel(repository: repository)
            }
        }
    }
}
