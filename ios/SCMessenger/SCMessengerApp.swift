//
//  SCMessengerApp.swift
//  SCMessenger
//
//  Main application entry point
//

import SwiftUI

@main
struct SCMessengerApp: App {
    // Repository - single source of truth
    @State private var meshRepository = MeshRepository()
    
    // Background service
    @State private var backgroundService: MeshBackgroundService?
    
    // Onboarding state
    @State private var hasCompletedOnboarding = UserDefaults.standard.bool(forKey: "hasCompletedOnboarding")
    
    init() {
        // Initialize background service after repository
        // Will be set in onAppear
    }
    
    var body: some Scene {
        WindowGroup {
            Group {
                if hasCompletedOnboarding {
                    MainTabView()
                } else {
                    OnboardingFlow()
                }
            }
            .environment(meshRepository)
            .onAppear {
                setupApp()
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.didEnterBackgroundNotification)) { _ in
                handleEnteringBackground()
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
                handleEnteringForeground()
            }
            .onChange(of: UserDefaults.standard.bool(forKey: "hasCompletedOnboarding")) { _, newValue in
                hasCompletedOnboarding = newValue
            }
        }
    }
    
    private func setupApp() {
        // Initialize background service
        backgroundService = MeshBackgroundService(meshRepository: meshRepository)
        backgroundService?.registerBackgroundTasks()
        
        // Initialize repository
        do {
            try meshRepository.initialize()
        } catch {
            print("❌ Failed to initialize repository: \(error)")
        }
    }
    
    private func handleEnteringBackground() {
        backgroundService?.onEnteringBackground()
    }
    
    private func handleEnteringForeground() {
        backgroundService?.onEnteringForeground()
    }
}
