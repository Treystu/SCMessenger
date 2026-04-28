//
//  NotificationGuidanceView.swift
//  SCMessenger
//
//  UI view for guiding users to enable notifications in Settings
//

import SwiftUI

/// View to guide users when notification permissions are denied
struct NotificationGuidanceView: View {
    @Environment(\.presentationMode) private var presentationMode

    private let titleText = "Notifications Disabled"
    private let messageText = "Please enable notifications in Settings to receive messages, alerts, and updates from SCMessenger."

    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: "bell.slash.fill")
                .resizable()
                .frame(width: 64, height: 64)
                .foregroundColor(.secondary)

            VStack(alignment: .leading, spacing: 8) {
                Text(titleText)
                    .font(.title2)
                    .fontWeight(.bold)
                    .multilineTextAlignment(.leading)

                Text(messageText)
                    .font(.body)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.leading)
            }

            Button(action: openSettings) {
                HStack {
                    Image(systemName: "gearshape")
                    Text("Open Settings")
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(Color.blue)
                .foregroundColor(.white)
                .cornerRadius(10)
            }

            Button(action: dismiss) {
                Text("Maybe Later")
                    .foregroundColor(.primary)
            }
            .padding(.top, 8)
        }
        .padding()
        .navigationTitle("Enable Notifications")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            NotificationLogger.shared.log("Notification Guidance View displayed")
        }
    }

    private func openSettings() {
        NotificationLogger.shared.log("Opening Settings from guidance view")

        if let url = URL(string: UIApplication.openSettingsURLString),
           UIApplication.shared.canOpenURL(url) {
            UIApplication.shared.open(url) { [weak self] success in
                if success {
                    NotificationLogger.shared.log("Settings opened successfully")
                } else {
                    NotificationLogger.shared.log("Failed to open Settings")
                }
                self?.presentationMode.wrappedValue.dismiss()
            }
        } else {
            NotificationLogger.shared.log("Cannot open Settings URL")
        }
    }

    private func dismiss() {
        NotificationLogger.shared.log("Notification Guidance View dismissed")
        presentationMode.wrappedValue.dismiss()
    }
}

#if DEBUG
struct NotificationGuidanceView_Previews: PreviewProvider {
    static var previews: some View {
        NavigationView {
            NotificationGuidanceView()
        }
    }
}
#endif
