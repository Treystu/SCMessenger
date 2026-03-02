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
    
    init() {
        // Initialize background service after repository
        // Will be set in onAppear
    }
    
    var body: some Scene {
        WindowGroup {
            MainTabView()
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
        }
    }
    
    private func setupApp() {
        // Initialize background service
        backgroundService = MeshBackgroundService(meshRepository: meshRepository)
        backgroundService?.registerBackgroundTasks()
        
        // Initialize + start repository so identity/service state is hydrated at launch.
        do {
            try meshRepository.initialize()
            meshRepository.start()
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
