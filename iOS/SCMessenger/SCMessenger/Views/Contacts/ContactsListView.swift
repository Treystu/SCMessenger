//
//  ContactsListView.swift
//  SCMessenger
//
//  Contacts list view
//

import SwiftUI
import VisionKit
import Vision

struct ContactsListView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var viewModel: ContactsViewModel?
    @State private var showingAddContact = false
    @State private var pendingChatConversation: Conversation?
    @State private var navigateToPendingChat = false
    @State private var nearbyPrefilledPeer: NearbyPeer? = nil

    var body: some View {
        List {
            // MARK: Nearby Peers — discovered on the mesh, not yet added
            let nearby = viewModel?.nearbyPeers ?? []
            if !nearby.isEmpty {
                Section {
                    ForEach(nearby) { peer in
                        NearbyPeerRow(peer: peer) {
                            nearbyPrefilledPeer = peer
                            showingAddContact = true
                        }
                    }
                } header: {
                    Label("Nearby", systemImage: "antenna.radiowaves.left.and.right")
                        .foregroundStyle(Color.accentColor)
                }
            }

            // MARK: Saved contacts
            Section {
                ForEach(viewModel?.filteredContacts ?? [], id: \.peerId) { contact in
                    NavigationLink(value: Conversation(peerId: contact.peerId, peerNickname: contact.nickname ?? "Unknown")) {
                        ContactRow(contact: contact)
                    }
                }
                .onDelete { offsets in
                    viewModel?.deleteContacts(at: offsets)
                }
            } header: {
                if !(viewModel?.filteredContacts ?? []).isEmpty {
                    Text("Contacts")
                }
            }
        }
        .navigationDestination(for: Conversation.self) { conversation in
            ChatView(conversation: conversation)
        }
        .navigationDestination(isPresented: $navigateToPendingChat) {
            if let conversation = pendingChatConversation {
                ChatView(conversation: conversation)
            }
        }
        .searchable(text: Binding(
            get: { viewModel?.searchText ?? "" },
            set: { viewModel?.searchText = $0 }
        ))
        .navigationTitle("Contacts")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    nearbyPrefilledPeer = nil
                    showingAddContact = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showingAddContact, onDismiss: {
            nearbyPrefilledPeer = nil
            viewModel?.loadContacts()
            if pendingChatConversation != nil {
                navigateToPendingChat = true
            }
        }) {
            AddContactView(
                pendingChatConversation: $pendingChatConversation,
                prefilledPeer: nearbyPrefilledPeer
            )
        }
        .onAppear {
            if viewModel == nil {
                viewModel = ContactsViewModel(repository: repository)
                viewModel?.loadContacts()
            }
        }
    }
}

// MARK: - Nearby Peer Row

struct NearbyPeerRow: View {
    let peer: NearbyPeer
    let onAdd: () -> Void

    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            ZStack {
                Circle()
                    .fill(Color.green.opacity(0.15))
                    .frame(width: 44, height: 44)
                Image(systemName: "dot.radiowaves.left.and.right")
                    .foregroundStyle(Color.green)
            }

            VStack(alignment: .leading, spacing: 2) {
                Text(peer.displayName)
                    .font(Theme.titleMedium)
                if peer.hasFullIdentity {
                    Text("● Identity verified")
                        .font(.system(.caption2))
                        .foregroundStyle(Color.green)
                } else {
                    Text(peer.peerId.prefix(16))
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(Theme.onSurfaceVariant)
                }
            }

            Spacer()

            Button {
                onAdd()
            } label: {
                Label("Add", systemImage: "person.badge.plus")
                    .labelStyle(.iconOnly)
                    .font(.system(size: 20))
                    .foregroundStyle(Color.accentColor)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 4)
    }
}

struct ContactRow: View {
    let contact: Contact

    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Circle()
                .fill(Theme.primaryContainer)
                .frame(width: 44, height: 44)
                .overlay {
                    Text((contact.nickname ?? "?").prefix(1).uppercased())
                        .font(Theme.titleMedium)
                        .foregroundStyle(Theme.onPrimaryContainer)
                }

            VStack(alignment: .leading, spacing: 4) {
                Text(contact.nickname ?? "Unknown")
                    .font(Theme.titleMedium)

                Text(contact.peerId.prefix(8))
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(Theme.onSurfaceVariant)
            }
        }
        .padding(.vertical, 4)
    }
}

struct AddContactView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(MeshRepository.self) private var repository
    @Binding var pendingChatConversation: Conversation?

    /// Pre-fill from a nearby peer when launched from the Nearby section.
    var prefilledPeer: NearbyPeer? = nil

    @State private var nickname = ""
    @State private var publicKey = ""
    @State private var peerId = ""
    @State private var listeners: [String] = []
    @State private var libp2pPeerId: String = ""
    @State private var error: String?
    @State private var showingQrScanner = false

    private var canUseQrScanner: Bool {
        if #available(iOS 16.0, *) {
            return DataScannerViewController.isSupported && DataScannerViewController.isAvailable
        }
        return false
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("Contact Information") {
                    Button(action: pasteIdentity) {
                        Label("Paste Identity Export", systemImage: "doc.on.clipboard")
                    }

                    Button(action: { showingQrScanner = true }) {
                        Label("Scan Contact QR", systemImage: "qrcode.viewfinder")
                    }
                    .disabled(!canUseQrScanner)
                    if !canUseQrScanner {
                        Text("QR scanning is unavailable on this device.")
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                    }

                    TextField("Nickname", text: $nickname)
                    TextField("Public Key", text: $publicKey)
                        .font(.system(.body, design: .monospaced))
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                    if !peerId.isEmpty {
                        Text("ID: \(peerId)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                if let error = error {
                    Section {
                        Text(error)
                            .foregroundStyle(.red)
                            .font(Theme.bodySmall)
                    }
                }

                Section {
                    Button("Add Contact") {
                        addContact(andChat: false)
                    }
                    .disabled(nickname.isEmpty || publicKey.isEmpty)

                    Button("Add & Chat") {
                        addContact(andChat: true)
                    }
                    .disabled(nickname.isEmpty || publicKey.isEmpty)
                }
            }
            .navigationTitle("Add Contact")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                if let peer = prefilledPeer, peerId.isEmpty {
                    peerId = peer.peerId
                    if let pk = peer.publicKey { publicKey = pk }
                    if let nick = peer.nickname, !nick.isEmpty { nickname = nick }
                    if let lpid = peer.libp2pPeerId, !lpid.isEmpty { libp2pPeerId = lpid }
                    listeners = peer.listeners
                }
            }
            .sheet(isPresented: $showingQrScanner) {
                if canUseQrScanner {
                    ContactQrScannerSheet(
                        onScan: { payload in
                            showingQrScanner = false
                            importQrPayload(payload)
                        },
                        onFailure: { message in
                            error = message
                        }
                    )
                } else {
                    Text("QR scanning is unavailable on this device.")
                        .padding()
                }
            }
        }
    }

    private func pasteIdentity() {
        guard let string = UIPasteboard.general.string else { return }

        guard let data = string.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            error = "Invalid format"
            return
        }

        if let key = json["public_key"] as? String { publicKey = key.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let nick = json["nickname"] as? String { nickname = nick.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let pid = json["identity_id"] as? String { peerId = pid.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let lpid = json["libp2p_peer_id"] as? String, !lpid.isEmpty { libp2pPeerId = lpid.trimmingCharacters(in: .whitespacesAndNewlines) }
        if let list = json["listeners"] as? [String] {
            listeners = list.map { $0.replacingOccurrences(of: " (Potential)", with: "") }
        }

        error = nil
    }

    private func addContact(andChat: Bool) {
        let finalPublicKey = publicKey.trimmingCharacters(in: .whitespacesAndNewlines)
        var finalPeerId = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        if finalPeerId.isEmpty { finalPeerId = String(finalPublicKey.prefix(16)) }

        if finalPublicKey.isEmpty {
            self.error = "Public key cannot be empty"
            return
        }
        if finalPublicKey.count != 64 {
            self.error = "Public key must be exactly 64 hex characters (got \(finalPublicKey.count))"
            return
        }
        let hexCharacterSet = CharacterSet(charactersIn: "0123456789abcdefABCDEF")
        if !finalPublicKey.unicodeScalars.allSatisfy({ hexCharacterSet.contains($0) }) {
            self.error = "Public key contains invalid characters (must be hex: 0-9, a-f)"
            return
        }

        // Store libp2p PeerId + listeners in notes for sendMessage routing
        var notesValue: String? = nil
        if !libp2pPeerId.isEmpty {
            let addrs = listeners.joined(separator: ",")
            notesValue = "libp2p_peer_id:\(libp2pPeerId);listeners:\(addrs)"
        }
        let contact = Contact(
            peerId: finalPeerId,
            nickname: nickname,
            publicKey: finalPublicKey,
            addedAt: UInt64(Date().timeIntervalSince1970),
            lastSeen: nil,
            notes: notesValue
        )

        do {
            try repository.addContact(contact)

            if !listeners.isEmpty {
                let peerIdForDial = libp2pPeerId.isEmpty ? finalPeerId : libp2pPeerId
                repository.connectToPeer(peerIdForDial, addresses: listeners)
            }

            if andChat {
                pendingChatConversation = Conversation(peerId: finalPeerId, peerNickname: nickname)
            } else {
                pendingChatConversation = nil
            }

            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func importQrPayload(_ raw: String) {
        guard let data = raw.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            error = "Invalid QR format"
            return
        }

        if let key = json["public_key"] as? String ?? json["publicKey"] as? String {
            publicKey = key.trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let nick = json["nickname"] as? String {
            nickname = nick.trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let pid = json["identity_id"] as? String ?? json["peerId"] as? String {
            peerId = pid.trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let lpid = json["libp2p_peer_id"] as? String, !lpid.isEmpty {
            libp2pPeerId = lpid.trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let list = json["listeners"] as? [String] {
            listeners = list.map { $0.replacingOccurrences(of: " (Potential)", with: "") }
        }
        error = nil
    }
}

@available(iOS 16.0, *)
private struct ContactQrScannerSheet: UIViewControllerRepresentable {
    var onScan: (String) -> Void
    var onFailure: (String) -> Void

    func makeUIViewController(context: Context) -> DataScannerViewController {
        let controller = DataScannerViewController(
            recognizedDataTypes: [.barcode(symbologies: [.qr])],
            qualityLevel: .balanced,
            recognizesMultipleItems: false,
            isHighFrameRateTrackingEnabled: false,
            isHighlightingEnabled: true
        )
        controller.delegate = context.coordinator
        return controller
    }

    func updateUIViewController(_ uiViewController: DataScannerViewController, context: Context) {
        do {
            try uiViewController.startScanning()
        } catch {
            onFailure("Unable to start camera scanner: \(error.localizedDescription)")
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(onScan: onScan)
    }

    final class Coordinator: NSObject, DataScannerViewControllerDelegate {
        private let onScan: (String) -> Void

        init(onScan: @escaping (String) -> Void) {
            self.onScan = onScan
        }

        func dataScanner(
            _ dataScanner: DataScannerViewController,
            didTapOn item: RecognizedItem
        ) {
            if case let .barcode(barcode) = item, let payload = barcode.payloadStringValue {
                onScan(payload)
            }
        }
    }
}
