//! Non-interactive self-test behind `scmessenger-cli test` (#651).
//!
//! The self-test is the documented smoke check for a fresh build, so it has to
//! run to completion in a non-interactive shell. It used to live inline in
//! `cmd_test`, where the third identity of the final stage was created with no
//! consent grant. `IronCore::initialize_identity` returns
//! `IronCoreError::ConsentRequired` in that state, so the command printed three
//! `[OK]` lines and then aborted with a bare `Error: Consent required` and exit
//! code 1 - no stage name, no remediation. That is the #377 failure mode, and
//! #651 reproduced it on current main.
//!
//! Two properties keep it from closing silently again:
//!
//! - every `IronCore` the probe creates comes from [`consented_core`], so a new
//!   stage cannot forget the grant;
//! - a failure carries the stage it failed in, so the command can name it.
//!
//! Nothing here touches the filesystem or the network: every core is in-memory,
//! which is what makes the probe testable as a plain unit of work. The probe
//! lives in the library rather than in `main.rs` because the CLI's binary target
//! sets `test = false`, so a unit test there would never run.

use anyhow::{anyhow, Result};

use scmessenger_core::message::MessageType;
use scmessenger_core::IronCore;

/// One stage of the self-test, in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    IdentityGeneration,
    MessageEncryption,
    MessageDecryption,
    EncryptionSecurity,
}

impl Stage {
    /// Stage name as printed by `scmessenger-cli test`.
    pub fn label(self) -> &'static str {
        match self {
            Stage::IdentityGeneration => "Identity generation",
            Stage::MessageEncryption => "Message encryption",
            Stage::MessageDecryption => "Message decryption",
            Stage::EncryptionSecurity => "Encryption security",
        }
    }

    /// What an operator can do about a failure in this stage.
    ///
    /// Every stage runs in memory, with no configuration, storage or network
    /// dependency, so no operator action can fix one: a failure here is a defect
    /// in the build. Saying that plainly is the remediation.
    pub fn remediation(self) -> &'static str {
        "no operator action applies: this stage runs in memory only, so a failure here is a defect in this build - report it with the stage name above"
    }
}

/// A stage that completed, with whatever detail the stage reports.
#[derive(Debug, Clone)]
pub struct StageOutcome {
    pub stage: Stage,
    /// Extra detail printed after the stage label; empty when there is none.
    pub detail: String,
}

impl StageOutcome {
    fn new(stage: Stage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
        }
    }
}

/// A stage that did not complete, carrying the stage so the caller can name it.
#[derive(Debug)]
pub struct Failure {
    pub stage: Stage,
    pub error: anyhow::Error,
}

impl Failure {
    pub fn new(stage: Stage, error: anyhow::Error) -> Self {
        Self { stage, error }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "self-test stage '{}' failed: {}",
            self.stage.label(),
            self.error
        )
    }
}

/// An `IronCore` that may initialize an identity.
///
/// Self-tests are non-interactive, so consent is granted programmatically - the
/// same thing every other CLI command path does before initializing. Routing
/// every construction through here is what keeps a stage from missing it.
fn consented_core() -> IronCore {
    let core = IronCore::new();
    core.grant_consent();
    core
}

fn stage_error(stage: Stage, context: &str, error: impl std::fmt::Debug) -> Failure {
    Failure::new(stage, anyhow!("{}: {:?}", context, error))
}

/// Run every self-test stage in order.
///
/// Returns one outcome per completed stage. On failure the returned [`Failure`]
/// names the stage, so no stage can fail anonymously.
pub fn run() -> Result<Vec<StageOutcome>, Failure> {
    const IDENTITY: Stage = Stage::IdentityGeneration;
    const ENCRYPT: Stage = Stage::MessageEncryption;
    const DECRYPT: Stage = Stage::MessageDecryption;
    const SECURITY: Stage = Stage::EncryptionSecurity;

    let alice = consented_core();
    let bob = consented_core();

    alice
        .initialize_identity()
        .map_err(|e| stage_error(IDENTITY, "Alice's identity failed to initialize", e))?;
    bob.initialize_identity()
        .map_err(|e| stage_error(IDENTITY, "Bob's identity failed to initialize", e))?;

    let bob_key = bob
        .get_identity_info()
        .public_key_hex
        .ok_or_else(|| Failure::new(IDENTITY, anyhow!("Bob's identity has no public key")))?;

    let mut stages = vec![StageOutcome::new(IDENTITY, "")];

    let envelope = alice
        .prepare_message(
            bob_key.clone(),
            "Test message".to_string(),
            MessageType::Text,
            None,
        )
        .map_err(|e| stage_error(ENCRYPT, "preparing a message for Bob failed", e))?;
    stages.push(StageOutcome::new(
        ENCRYPT,
        format!("{} bytes", envelope.envelope_data.len()),
    ));

    let decrypted = bob
        .receive_message(envelope.envelope_data)
        .map_err(|e| stage_error(DECRYPT, "Bob could not decrypt the message", e))?;
    let text = decrypted
        .text_content()
        .ok_or_else(|| Failure::new(DECRYPT, anyhow!("the decrypted message carried no text")))?;
    if text != "Test message" {
        return Err(Failure::new(
            DECRYPT,
            anyhow!("the decrypted text did not match the text that was sent"),
        ));
    }
    stages.push(StageOutcome::new(DECRYPT, ""));

    // The third identity is the whole point of the security stage: it must fail
    // to decrypt a message addressed to Bob. It needs the same consent grant as
    // the other two, which is the defect #651 records.
    let eve = consented_core();
    eve.initialize_identity()
        .map_err(|e| stage_error(SECURITY, "the third identity failed to initialize", e))?;

    let secret = alice
        .prepare_message(bob_key, "Secret".to_string(), MessageType::Text, None)
        .map_err(|e| stage_error(SECURITY, "preparing the security-stage message failed", e))?;
    if eve.receive_message(secret.envelope_data).is_ok() {
        return Err(Failure::new(
            SECURITY,
            anyhow!("a third identity decrypted a message addressed to Bob"),
        ));
    }
    stages.push(StageOutcome::new(SECURITY, ""));

    Ok(stages)
}
