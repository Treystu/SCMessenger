//
//  OnboardingFlow.swift
//  SCMessenger
//
//  Onboarding flow with 5 steps
//

import SwiftUI
import os

private let logger = Logger(subsystem: "com.scmessenger", category: "Onboarding")

struct OnboardingFlow: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel = OnboardingViewModel()
    
    var body: some View {
        TabView(selection: $viewModel.currentStep) {
            WelcomeView()
                .tag(0)
            
            IdentityView()
                .tag(1)
            
            PermissionsView()
                .tag(2)
            
            RelayExplanationView()
                .tag(3)
            
            CompletionView(viewModel: viewModel)
                .tag(4)
        }
        .tabViewStyle(.page)
        .indexViewStyle(.page(backgroundDisplayMode: .always))
        .environment(viewModel)
    }
}

struct WelcomeView: View {
    @Environment(OnboardingViewModel.self) private var viewModel
    
    var body: some View {
        VStack(spacing: Theme.spacingLarge) {
            Spacer()
            
            Image(systemName: "network")
                .font(.system(size: 80))
                .foregroundStyle(Theme.onPrimaryContainer)
            
            Text("Welcome to SCMessenger")
                .font(Theme.headlineLarge)
                .multilineTextAlignment(.center)
            
            Text("The world's first truly sovereign messenger")
                .font(Theme.bodyLarge)
                .foregroundStyle(Theme.onSurfaceVariant)
                .multilineTextAlignment(.center)
                .padding(.horizontal, Theme.spacingXLarge)
            
            Spacer()
            
            Button("Get Started") {
                viewModel.advance()
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        }
        .padding(Theme.spacingLarge)
    }
}

struct IdentityView: View {
    @Environment(MeshRepository.self) private var repository
    @Environment(OnboardingViewModel.self) private var viewModel
    @State private var isGenerating = false
    @State private var identity: IdentityInfo?
    @State private var nickname = ""
    
    var body: some View {
        VStack(spacing: Theme.spacingLarge) {
            Text("Your Identity")
                .font(Theme.headlineLarge)
            
            if let identity = identity {
                VStack(spacing: Theme.spacingMedium) {
                    Text("Identity Generated")
                        .font(Theme.titleMedium)
                    
                    Text(identity.publicKeyHex?.prefix(16) ?? "")
                        .font(.system(.body, design: .monospaced))
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
                .primaryContainerStyle()
            } else {
                Button {
                    generateIdentity()
                } label: {
                    if isGenerating {
                        ProgressView()
                    } else {
                        Label("Generate Identity", systemImage: "key.fill")
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(isGenerating)
            }

            TextField("Choose a nickname", text: $nickname)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .padding(12)
                .background(Theme.primaryContainer, in: RoundedRectangle(cornerRadius: 12))
            
            Spacer()
            
            Button("Continue") {
                viewModel.advance()
            }
            .buttonStyle(.borderedProminent)
            .disabled(identity == nil || nickname.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        }
        .padding(Theme.spacingLarge)
        .onAppear {
            if identity == nil {
                identity = repository.getIdentityInfo()
            }
            if nickname.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
               let existing = repository.getIdentityInfo()?.nickname,
               !existing.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                nickname = existing
            }
        }
    }
    
    private func generateIdentity() {
        Task {
            isGenerating = true
            defer { isGenerating = false }
            
            do {
                try repository.createIdentity()
                let trimmedNickname = nickname.trimmingCharacters(in: .whitespacesAndNewlines)
                if !trimmedNickname.isEmpty {
                    try repository.setNickname(trimmedNickname)
                }
                identity = repository.getIdentityInfo()
            } catch {
                logger.error("Failed to generate identity: \(error.localizedDescription)")
                // Keep identity as nil to show error state
            }
        }
    }
}

struct PermissionsView: View {
    @Environment(OnboardingViewModel.self) private var viewModel
    
    var body: some View {
        VStack(spacing: Theme.spacingLarge) {
            Text("Permissions")
                .font(Theme.headlineLarge)
            
            VStack(alignment: .leading, spacing: Theme.spacingMedium) {
                PermissionRow(icon: "antenna.radiowaves.left.and.right", title: "Bluetooth", description: "Required for mesh networking")
                PermissionRow(icon: "wifi", title: "Local Network", description: "Enables WiFi Direct connections")
                PermissionRow(icon: "bell.fill", title: "Notifications", description: "Get notified of new messages")
            }
            
            Spacer()
            
            Button("Continue") {
                viewModel.advance()
            }
            .buttonStyle(.borderedProminent)
        }
        .padding(Theme.spacingLarge)
    }
}

struct PermissionRow: View {
    let icon: String
    let title: String
    let description: String
    
    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(Theme.onPrimaryContainer)
                .frame(width: 40)
            
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(Theme.titleMedium)
                Text(description)
                    .font(Theme.bodySmall)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }
        }
    }
}

struct RelayExplanationView: View {
    @Environment(OnboardingViewModel.self) private var viewModel
    
    var body: some View {
        VStack(spacing: Theme.spacingLarge) {
            Image(systemName: "arrow.triangle.2.circlepath")
                .font(.system(size: 60))
                .foregroundStyle(Theme.onErrorContainer)
            
            Text("Relay = Messaging")
                .font(Theme.headlineLarge)
            
            VStack(alignment: .leading, spacing: Theme.spacingSmall) {
                BulletPoint("You relay messages for others")
                BulletPoint("Others relay messages for you")
                BulletPoint("No relay means no messaging")
                BulletPoint("This is how the mesh stays strong")
            }
            .errorContainerStyle()
            
            Spacer()
            
            Button("I Understand") {
                viewModel.advance()
            }
            .buttonStyle(.borderedProminent)
        }
        .padding(Theme.spacingLarge)
    }
}

struct BulletPoint: View {
    let text: String
    
    init(_ text: String) {
        self.text = text
    }
    
    var body: some View {
        HStack(alignment: .top, spacing: Theme.spacingSmall) {
            Text("•")
            Text(text)
                .font(Theme.bodyMedium)
        }
    }
}

struct CompletionView: View {
    let viewModel: OnboardingViewModel
    
    var body: some View {
        VStack(spacing: Theme.spacingLarge) {
            Spacer()
            
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 80))
                .foregroundStyle(.green)
            
            Text("You're All Set!")
                .font(Theme.headlineLarge)
            
            Text("Start messaging on the mesh")
                .font(Theme.bodyLarge)
                .foregroundStyle(Theme.onSurfaceVariant)
            
            Spacer()
            
            Button("Start Messaging") {
                viewModel.completeOnboarding()
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        }
        .padding(Theme.spacingLarge)
    }
}
