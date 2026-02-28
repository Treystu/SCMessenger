//
//  SettingsView.swift
//  SCMessenger
//
//  Main settings view - Full feature parity with Android SettingsScreen.kt
//  Mirrors: android/.../ui/screens/SettingsScreen.kt
//

import SwiftUI
import CoreImage.CIFilterBuiltins

struct SettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: SettingsViewModel?
    @State private var showingIdentityQr = false
    @State private var showingResetConfirmation = false
    @AppStorage("hasCompletedOnboarding") private var hasCompletedOnboarding = false

    var body: some View {
        Form {
            // MARK: - Service Control (mirrors Android)
            Section {
                HStack {
                    Image(systemName: viewModel?.isServiceRunning == true ? "antenna.radiowaves.left.and.right" : "antenna.radiowaves.left.and.right.slash")
                        .foregroundStyle(viewModel?.isServiceRunning == true ? Theme.onPrimaryContainer : Theme.onSurfaceVariant)
                    Text("Mesh Service")
                        .font(Theme.titleMedium.weight(.medium))
                    Spacer()
                    Text(viewModel?.isServiceRunning == true ? "Running" : "Stopped")
                        .font(Theme.bodyMedium)
                        .foregroundStyle(viewModel?.isServiceRunning == true ? .green : Theme.onSurfaceVariant)
                }

                Button {
                    if viewModel?.isServiceRunning == true {
                        viewModel?.stopService()
                    } else {
                        viewModel?.startService()
                    }
                } label: {
                    HStack {
                        Image(systemName: viewModel?.isServiceRunning == true ? "stop.circle.fill" : "play.circle.fill")
                        Text(viewModel?.isServiceRunning == true ? "Stop Service" : "Start Service")
                    }
                }
            } header: {
                Text("Service Control")
            }

            // MARK: - Relay & Messaging
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

            // MARK: - Advanced
            Section {
                NavigationLink("Mesh Settings") {
                    MeshSettingsView()
                }

                NavigationLink("Privacy Settings") {
                    PrivacySettingsView()
                }

                NavigationLink("Power Settings") {
                    PowerSettingsView()
                }

                NavigationLink("Diagnostics") {
                    DiagnosticsView()
                }
            } header: {
                Text("Advanced")
            }

            // MARK: - App Preferences (mirrors Android)
            Section {
                Toggle("Notifications", isOn: Binding(
                    get: { viewModel?.isNotificationsEnabled ?? true },
                    set: { viewModel?.isNotificationsEnabled = $0 }
                ))
            } header: {
                Text("App Preferences")
            }

            // MARK: - Identity
            Section {
                HStack {
                    Text("Nickname")
                    Spacer()
                    TextField("Enter nickname", text: Binding(
                        get: { viewModel?.nickname ?? "" },
                        set: { viewModel?.updateNickname($0) }
                    ))
                    .multilineTextAlignment(.trailing)
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.words)
                }

                HStack {
                    Text("Identity ID")
                    Spacer()
                    Text(repository.getIdentitySnippet())
                        .font(.system(.body, design: .monospaced))
                        .foregroundStyle(Theme.onSurfaceVariant)
                }

                Button {
                    if let id = repository.getFullIdentityInfo()?.identityId {
                        UIPasteboard.general.string = id
                    }
                } label: {
                    Label("Copy Identity ID", systemImage: "doc.on.doc")
                }

                Button {
                    if let key = repository.getFullIdentityInfo()?.publicKeyHex {
                        UIPasteboard.general.string = key
                    }
                } label: {
                    Label("Copy Public Key", systemImage: "key")
                }

                Button {
                    if let export = viewModel?.getIdentityExportString() {
                        UIPasteboard.general.string = export
                    }
                } label: {
                    Label("Copy Full Identity Export", systemImage: "square.and.arrow.up")
                }

                Button {
                    showingIdentityQr = true
                } label: {
                    Label("Show Identity QR", systemImage: "qrcode")
                }
            } header: {
                Text("Identity")
            }

            // MARK: - Information (mirrors Android)
            Section {
                HStack {
                    Text("Contacts")
                    Spacer()
                    Text("\(viewModel?.getContactCount() ?? 0)")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }

                HStack {
                    Text("Messages")
                    Spacer()
                    Text("\(viewModel?.getMessageCount() ?? 0)")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }

                HStack {
                    Text("Version")
                    Spacer()
                    Text("0.1.2")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            } header: {
                Text("Information")
            }

            // MARK: - Danger Zone (mirrors Android Delete All Data section)
            Section {
                Button(role: .destructive) {
                    showingResetConfirmation = true
                } label: {
                    Label("Delete All Data & Reset App", systemImage: "trash.fill")
                }
            } header: {
                Text("Danger Zone")
            } footer: {
                Text("Permanently deletes your identity, contacts, messages, and all app settings. This cannot be undone.")
                    .foregroundStyle(.secondary)
            }
        }
        .navigationTitle("Settings")
        .onAppear {
            if viewModel == nil {
                viewModel = SettingsViewModel(repository: repository)
                viewModel?.loadSettings()
            }
        }
        .sheet(isPresented: $showingIdentityQr) {
            IdentityQrSheet(payload: viewModel?.getIdentityExportString() ?? "{}")
        }
        .confirmationDialog(
            "Delete All Data & Reset App?",
            isPresented: $showingResetConfirmation,
            titleVisibility: .visible
        ) {
            Button("Delete All Data", role: .destructive) {
                viewModel?.resetAllData()
                hasCompletedOnboarding = false
            }
            Button("Cancel", role: .cancel) { }
        } message: {
            Text("This will permanently delete your identity, all contacts, messages, and settings. You will need to set up the app again.")
        }
    }
}

// MARK: - Relay Components

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

private struct IdentityQrSheet: View {
    let payload: String
    @Environment(\.dismiss) private var dismiss
    private let context = CIContext()
    private let filter = CIFilter.qrCodeGenerator()

    var body: some View {
        NavigationStack {
            VStack(spacing: Theme.spacingMedium) {
                if let image = qrImage(from: payload) {
                    Image(uiImage: image)
                        .interpolation(.none)
                        .resizable()
                        .scaledToFit()
                        .frame(maxWidth: 320, maxHeight: 320)
                } else {
                    Text("Unable to generate QR code.")
                        .foregroundStyle(.secondary)
                }

                Text("Scan to add this contact with full identity export details.")
                    .font(Theme.bodySmall)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, Theme.spacingLarge)
            }
            .padding(Theme.spacingLarge)
            .navigationTitle("Identity QR")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }

    private func qrImage(from string: String) -> UIImage? {
        let data = Data(string.utf8)
        filter.setValue(data, forKey: "inputMessage")
        filter.setValue("Q", forKey: "inputCorrectionLevel")

        guard let outputImage = filter.outputImage else { return nil }
        let scaled = outputImage.transformed(by: CGAffineTransform(scaleX: 12, y: 12))
        guard let cgImage = context.createCGImage(scaled, from: scaled.extent) else { return nil }
        return UIImage(cgImage: cgImage)
    }
}

// MARK: - Mesh Settings (mirrors Android MeshSettingsScreen.kt)

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
            // Transport Settings
            Section("Transports") {
                Toggle("Bluetooth LE", isOn: Binding(
                    get: { viewModel?.settings?.bleEnabled ?? true },
                    set: { viewModel?.updateBleEnabled($0) }
                ))

                Toggle("WiFi Aware", isOn: Binding(
                    get: { viewModel?.settings?.wifiAwareEnabled ?? false },
                    set: { viewModel?.updateWifiAwareEnabled($0) }
                ))

                Toggle("WiFi Direct", isOn: Binding(
                    get: { viewModel?.settings?.wifiDirectEnabled ?? false },
                    set: { viewModel?.updateWifiDirectEnabled($0) }
                ))

                Toggle("Internet / Swarm", isOn: Binding(
                    get: { viewModel?.settings?.internetEnabled ?? true },
                    set: { viewModel?.updateInternetEnabled($0) }
                ))
            }

            // Discovery Mode
            Section("Discovery") {
                Picker("Mode", selection: discoveryModeBinding) {
                    Text("Aggressive (Normal)").tag(DiscoveryMode.normal)
                    Text("Balanced (Cautious)").tag(DiscoveryMode.cautious)
                    Text("Passive (Paranoid)").tag(DiscoveryMode.paranoid)
                }
            }



            // Battery Floor
            Section("Battery") {
                VStack(alignment: .leading) {
                    HStack {
                        Text("Minimum Level")
                        Spacer()
                        Text("\(viewModel?.settings?.batteryFloor ?? 20)%")
                            .foregroundStyle(Theme.onSurfaceVariant)
                    }
                    Slider(
                        value: Binding(
                            get: { Double(viewModel?.settings?.batteryFloor ?? 20) },
                            set: { viewModel?.updateBatteryFloor(UInt8($0)) }
                        ),
                        in: 5...50,
                        step: 5
                    )
                    .tint(Theme.onPrimaryContainer)
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

// MARK: - Privacy Settings (mirrors Android PrivacySettingsScreen.kt)

struct PrivacySettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: SettingsViewModel?
    @State private var isRotationEnabled = true

    var body: some View {
        Form {
            // Privacy by Design Notice (mirrors Android)
            Section {
                VStack(alignment: .leading, spacing: Theme.spacingSmall) {
                    HStack(spacing: Theme.spacingSmall) {
                        Image(systemName: "lock.shield.fill")
                            .foregroundStyle(Theme.onPrimaryContainer)
                        Text("Privacy by Design")
                            .font(Theme.titleSmall.weight(.bold))
                    }
                    Text("SCMessenger is built with privacy at its core. All messages are end-to-end encrypted. These settings provide additional privacy protections.")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
                .padding(.vertical, Theme.spacingXSmall)
            }

            // Onion Routing (mirrors Android)
            Section("Onion Routing") {
                Toggle("Enable Onion Routing", isOn: Binding(
                    get: { viewModel?.settings?.onionRouting ?? false },
                    set: { viewModel?.updateOnionRouting($0) }
                ))

                Text("Route messages through multiple hops to obscure sender and receiver. This makes it harder to trace who is communicating with whom.")
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }

            // BLE Identity Rotation
            Section("BLE Identity Rotation") {
                Toggle("Rotate BLE Identity", isOn: Binding(
                    get: { viewModel?.isBleRotationEnabled ?? true },
                    set: { val in
                        viewModel?.toggleBleRotation(enabled: val)
                        isRotationEnabled = val
                    }
                ))

                HStack {
                    Text("Rotation Interval")
                    Spacer()
                    Text("\(Int((viewModel?.bleRotationInterval ?? 900) / 60)) min")
                        .foregroundStyle(Theme.onSurfaceVariant)
                }

                Text("BLE identity rotation changes your device's Bluetooth advertising data periodically, making it harder for third parties to track your device over time.")
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }

            // Additional Privacy Features
            Section("Additional Privacy Features") {
                Toggle("Cover Traffic", isOn: Binding(
                    get: { viewModel?.settings?.coverTrafficEnabled ?? false },
                    set: { viewModel?.isCoverTrafficEnabled = $0 }
                ))
                Text("Send dummy traffic to resist traffic analysis")
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)

                Toggle("Message Padding", isOn: Binding(
                    get: { viewModel?.settings?.messagePaddingEnabled ?? false },
                    set: { viewModel?.isMessagePaddingEnabled = $0 }
                ))
                Text("Pad messages to hide actual message length")
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)

                Toggle("Timing Obfuscation", isOn: Binding(
                    get: { viewModel?.settings?.timingObfuscationEnabled ?? false },
                    set: { viewModel?.isTimingObfuscationEnabled = $0 }
                ))
                Text("Add random delays to obscure communication patterns")
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }

            // Privacy Best Practices (mirrors Android)
            Section("Privacy Best Practices") {
                VStack(alignment: .leading, spacing: Theme.spacingXSmall) {
                    BulletPoint("Enable onion routing for maximum anonymity")
                    BulletPoint("Use unique identities for different contexts")
                    BulletPoint("Be aware that metadata can still leak information")
                    BulletPoint("Consider physical security of your device")
                    BulletPoint("SCMessenger does not collect any user data or analytics")
                }
                .font(Theme.bodySmall)
            }
        }
        .navigationTitle("Privacy Settings")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            if viewModel == nil {
                viewModel = SettingsViewModel(repository: repository)
                viewModel?.loadSettings()
            }
        }
    }
}

// MARK: - Power Settings (NEW - mirrors Android PowerSettingsScreen.kt)

struct PowerSettingsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: SettingsViewModel?

    var body: some View {
        Form {
            // AutoAdjust Engine
            Section("AutoAdjust Engine") {
                Toggle("Enable AutoAdjust", isOn: Binding(
                    get: { viewModel?.isAutoAdjustEnabled ?? true },
                    set: { viewModel?.isAutoAdjustEnabled = $0 }
                ))

                Text("Automatically adjusts BLE scan intervals and relay capacity based on battery level, network conditions, and device usage patterns.")
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }

            // Power Saving Tips (mirrors Android)
            Section("Power Saving Tips") {
                VStack(alignment: .leading, spacing: Theme.spacingXSmall) {
                    BulletPoint("Enable AutoAdjust for optimal battery use")
                    BulletPoint("Reduce BLE scan frequency when not actively messaging")
                    BulletPoint("Lower relay budget to reduce background processing")
                    BulletPoint("Set a higher battery floor to preserve reserve power")
                    BulletPoint("Use WiFi over BLE when available for better efficiency")
                }
                .font(Theme.bodySmall)
            }
        }
        .navigationTitle("Power Settings")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            if viewModel == nil {
                viewModel = SettingsViewModel(repository: repository)
                viewModel?.loadSettings()
            }
        }
    }
}
