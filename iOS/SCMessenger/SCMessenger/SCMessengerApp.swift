//
//  SCMessengerApp.swift
//  SCMessenger
//
//  Main application entry point
//

import SwiftUI

@main
struct SCMessengerApp: App {
    private let installChoiceKey = "hasCompletedInstallModeChoice"

    // Repository - single source of truth
    @State private var meshRepository = MeshRepository()
    
    // Background service
    @State private var backgroundService: MeshBackgroundService?
    @State private var didRunSetup = false
    @State private var showOnboarding = false
    
    init() {
        // Initialize background service after repository
        // Will be set in onAppear
    }
    
    var body: some Scene {
        WindowGroup {
            Group {
                if showOnboarding {
                    OnboardingFlow {
                        showOnboarding = false
                    }
                } else {
                    MainTabView()
                }
            }
            .environment(meshRepository)
            .onAppear { setupApp() }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.didEnterBackgroundNotification)) { _ in
                handleEnteringBackground()
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
                handleEnteringForeground()
            }
        }
    }
    
    private func setupApp() {
        if didRunSetup { return }
        didRunSetup = true

        // Initialize background service
        backgroundService = MeshBackgroundService(meshRepository: meshRepository)
        backgroundService?.registerBackgroundTasks()
        
        // Initialize + start repository so identity/service state is hydrated at launch.
        do {
            try meshRepository.initialize()
            meshRepository.start()
            refreshOnboardingGate()
        } catch {
            print("❌ Failed to initialize repository: \(error)")
        }
    }
    
    private func handleEnteringBackground() {
        backgroundService?.onEnteringBackground()
    }
    
    private func handleEnteringForeground() {
        backgroundService?.onEnteringForeground()
        refreshOnboardingGate()
    }

    private func refreshOnboardingGate() {
        var installChoiceCompleted = UserDefaults.standard.bool(forKey: installChoiceKey)
        let hasIdentity = meshRepository.isIdentityInitialized()
        if hasIdentity && !installChoiceCompleted {
            UserDefaults.standard.set(true, forKey: installChoiceKey)
            UserDefaults.standard.set(true, forKey: "hasCompletedOnboarding")
            installChoiceCompleted = true
        }
        showOnboarding = !installChoiceCompleted && !hasIdentity
    }
}
