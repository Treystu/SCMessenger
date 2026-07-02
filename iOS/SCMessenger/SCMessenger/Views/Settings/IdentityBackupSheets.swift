//
//  IdentityBackupSheets.swift
//  SCMessenger
//
//  Export/import UI for the passphrase-encrypted identity backup (identity
//  key + ratchet sessions + contacts). Distinct from the plain-text
//  identity-card export/QR in SettingsView's IdentityQrSheet.
//  Mirrors: android/.../ui/screens/SettingsScreen.kt (export/import dialogs)
//

import SwiftUI

struct ExportIdentityBackupSheet: View {
    let viewModel: SettingsViewModel
    @Environment(\.dismiss) private var dismiss

    @State private var passphrase = ""
    @State private var confirmation = ""
    @State private var validationError: String?
    @State private var copiedMessage: String?

    var body: some View {
        NavigationStack {
            Form {
                if let result = viewModel.backupExportResult {
                    Section {
                        switch result {
                        case .success(let backup):
                            Text("Copy this and store it somewhere safe. Anyone with this string and your passphrase can restore your identity.")
                                .font(Theme.bodySmall)
                                .foregroundStyle(.secondary)
                            Text(backup)
                                .font(.system(.footnote, design: .monospaced))
                                .textSelection(.enabled)
                            Button {
                                UIPasteboard.general.string = backup
                                copiedMessage = "Backup copied to clipboard"
                            } label: {
                                Label("Copy Backup", systemImage: "doc.on.doc")
                            }
                            if let copiedMessage {
                                Text(copiedMessage)
                                    .font(Theme.bodySmall)
                                    .foregroundStyle(.green)
                            }
                        case .failure(let error):
                            Text("Failed to export backup: \(error.localizedDescription)")
                                .foregroundStyle(.red)
                        }
                    }
                } else {
                    Section {
                        Text("Choose a passphrase to encrypt your identity key, active conversations, and contacts. You'll need this exact passphrase to restore it later — write it down somewhere safe.")
                            .font(Theme.bodySmall)
                            .foregroundStyle(.secondary)
                        SecureField("Passphrase", text: $passphrase)
                        SecureField("Confirm passphrase", text: $confirmation)
                        if let validationError {
                            Text(validationError)
                                .font(Theme.bodySmall)
                                .foregroundStyle(.red)
                        }
                    }
                    Section {
                        Button(viewModel.isExportingBackup ? "Exporting…" : "Export Identity Backup") {
                            switch validateBackupPassphrase(passphrase, confirmation: confirmation) {
                            case .tooShort:
                                validationError = "Passphrase must be at least 8 characters"
                            case .mismatch:
                                validationError = "Passphrases don't match"
                            case .valid:
                                validationError = nil
                                viewModel.exportIdentityBackup(passphrase: passphrase)
                            }
                        }
                        .disabled(viewModel.isExportingBackup)
                    }
                }
            }
            .navigationTitle("Export Identity Backup")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") {
                        viewModel.clearBackupExportResult()
                        dismiss()
                    }
                }
            }
        }
    }
}

struct ImportIdentityBackupSheet: View {
    let viewModel: SettingsViewModel
    @Environment(\.dismiss) private var dismiss

    @State private var backupText = ""
    @State private var passphrase = ""

    var body: some View {
        NavigationStack {
            Form {
                if let result = viewModel.backupImportResult {
                    Section {
                        switch result {
                        case .success:
                            Text("Identity restored successfully. Your conversations and contacts should now be back.")
                                .font(Theme.bodySmall)
                                .foregroundStyle(.green)
                        case .failure(let error):
                            Text("Failed to restore identity: \(error.localizedDescription)")
                                .foregroundStyle(.red)
                        }
                    }
                } else {
                    Section {
                        TextField("Paste backup string", text: $backupText, axis: .vertical)
                            .lineLimit(3...6)
                        SecureField("Passphrase", text: $passphrase)
                    } footer: {
                        Text("Enter the exact passphrase used when this backup was exported.")
                    }
                    Section {
                        Button(viewModel.isImportingBackup ? "Importing…" : "Import") {
                            viewModel.importIdentityBackup(backup: backupText, passphrase: passphrase)
                        }
                        .disabled(
                            viewModel.isImportingBackup ||
                                backupText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ||
                                passphrase.isEmpty
                        )
                    }
                }
            }
            .navigationTitle("Import Identity Backup")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                if viewModel.backupImportResult != nil {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Done") {
                            viewModel.clearBackupImportResult()
                            dismiss()
                        }
                    }
                } else {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Cancel") { dismiss() }
                    }
                }
            }
        }
    }
}
