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
            ContentView()
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

// Placeholder ContentView - will be replaced with actual navigation
struct ContentView: View {
    @Environment(MeshRepository.self) private var repository
    
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("SCMessenger")
                .font(.largeTitle)
            Text("Sovereign Mesh Network")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .padding()
    }
}
