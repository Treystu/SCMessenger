//
//  BackupPassphraseValidator.swift
//  SCMessenger
//
//  Mirrors: android/.../utils/BackupPassphraseValidator.kt
//

import Foundation

/// Validation result for a user-chosen identity backup passphrase.
enum BackupPassphraseValidation: Equatable {
    case valid
    case tooShort
    case mismatch
}

private let minBackupPassphraseLength = 8

/// Validate a passphrase entered twice (entry + confirmation) before it's
/// used to encrypt an identity backup export. A typo here means an
/// unrecoverable backup, so both a minimum length and an exact match are
/// required.
func validateBackupPassphrase(_ passphrase: String, confirmation: String) -> BackupPassphraseValidation {
    if passphrase.count < minBackupPassphraseLength {
        return .tooShort
    }
    if passphrase != confirmation {
        return .mismatch
    }
    return .valid
}
