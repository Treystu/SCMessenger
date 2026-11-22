// Regression coverage for #651.
//
// `scmessenger-cli test` is the documented smoke check for a fresh build, so it
// must complete in a non-interactive shell. It used to abort after the
// "Message decryption" stage with `Error: Consent required`, because the third
// identity created for the encryption-security stage was initialized without
// the consent grant that every other path performs.
//
// This runs the real binary, the way an operator runs the smoke check, rather
// than re-implementing the sequence: re-implementing it would have granted
// consent in the test and never exercised the path that was broken.

use std::process::Command;

#[test]
fn built_in_self_test_completes_non_interactively() {
    let bin = env!("CARGO_BIN_EXE_scmessenger-cli");
    let output = Command::new(bin)
        .arg("test")
        .output()
        .expect("the self-test binary should be runnable");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "`scmessenger-cli test` must exit 0 in a non-interactive shell; status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr
    );

    for stage in [
        "Identity generation",
        "Message encryption",
        "Message decryption",
        "Encryption security",
    ] {
        assert!(
            stdout.contains(stage),
            "stage '{}' must be reached and reported; stdout:\n{}",
            stage,
            stdout
        );
    }

    assert!(
        stdout.contains("All tests passed!"),
        "the self-test must report completion; stdout:\n{}",
        stdout
    );

    assert!(
        !stdout.contains("Consent required") && !stderr.contains("Consent required"),
        "the self-test must not abort on consent; stdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}
