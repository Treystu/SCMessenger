//
//  JoinMeshView.swift
//  SCMessenger
//
//  View for joining mesh topics
//

import SwiftUI

struct JoinMeshView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(MeshRepository.self) private var repository
    
    @State private var topicManager: TopicManager?
    @State private var topicName = ""
    @State private var autoSubscribe = true
    @State private var error: String?
    
    var body: some View {
        NavigationStack {
            Form {
                Section("Join Mesh Topic") {
                    TextField("Topic Name", text: $topicName)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    
                    Toggle("Auto-subscribe to messages", isOn: $autoSubscribe)
                }
                
                if let error = error {
                    Section {
                        Text(error)
                            .foregroundStyle(.red)
                            .font(Theme.bodySmall)
                    }
                }
                
                Section {
                    Button("Join") {
                        joinMesh()
                    }
                    .disabled(topicName.isEmpty)
                }
                
                Section("Subscribed Meshes") {
                    ForEach(topicManager?.listTopics() ?? [], id: \.self) { topic in
                        HStack {
                            Text(topic)
                                .font(Theme.bodyMedium)
                            Spacer()
                            Button {
                                leaveTopic(topic)
                            } label: {
                                Image(systemName: "xmark.circle.fill")
                                    .foregroundStyle(.red)
                            }
                        }
                    }
                }
                
                Section {
                    Text("Topics allow you to join specific mesh networks and receive messages from all participants in that topic.")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            }
            .navigationTitle("Join Mesh")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                if topicManager == nil {
                    topicManager = TopicManager(meshRepository: repository)
                }
            }
        }
    }
    
    private func joinMesh() {
        do {
            try topicManager?.subscribe(to: topicName)
            error = nil
            topicName = ""
        } catch {
            self.error = error.localizedDescription
        }
    }
    
    private func leaveTopic(_ topic: String) {
        try? topicManager?.unsubscribe(from: topic)
    }
}
