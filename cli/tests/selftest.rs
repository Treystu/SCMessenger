//! Regression coverage for #651, and for the #377 failure mode it reproduces:
//! `scmessenger-cli test` must run to completion without interactive consent.
//!
//! The command is a binary subcommand and the CLI's binary target sets
//! `test = false`, so unit tests inside `main.rs` would never run. The probe
//! therefore lives in the library (`scmessenger_cli::selftest`) and these tests
//! sit on cargo's path, which is what stops the defect from closing silently a
//! third time.

use scmessenger_cli::selftest::{self, Stage};

#[test]
fn self_test_completes_non_interactively() {
    let stages = selftest::run().expect("the self-test must not require interactive consent");

    let order: Vec<Stage> = stages.iter().map(|outcome| outcome.stage).collect();
    assert_eq!(
        order,
        vec![
            Stage::IdentityGeneration,
            Stage::MessageEncryption,
            Stage::MessageDecryption,
            Stage::EncryptionSecurity,
        ],
        "every stage must run, in order, and report"
    );
}

#[test]
fn encryption_stage_reports_the_envelope_size() {
    let stages = selftest::run().expect("the self-test must run");
    let encryption = stages
        .iter()
        .find(|outcome| outcome.stage == Stage::MessageEncryption)
        .expect("the encryption stage must report");
    assert!(
        encryption.detail.ends_with(" bytes") && encryption.detail != "0 bytes",
        "envelope size must be reported, got {:?}",
        encryption.detail
    );
}

#[test]
fn a_failure_names_its_stage_and_offers_a_remediation() {
    let failure = selftest::Failure::new(
        Stage::EncryptionSecurity,
        anyhow::anyhow!("Consent required"),
    );

    let text = failure.to_string();
    assert!(
        text.contains("Encryption security"),
        "the failure must name its stage, got {:?}",
        text
    );
    assert!(text.contains("Consent required"));
    assert!(
        !Stage::EncryptionSecurity.remediation().is_empty(),
        "a failing stage must offer a remediation"
    );
}
