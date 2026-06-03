// Socket activation (systemd) integration.
//
// Detects whether the process was socket-activated by systemd and
// returns the activation status for the desktop bridge.
//
// References:
//   - https://www.freedesktop.org/software/systemd/man/sd_listen_fds.html
//   - https://www.freedesktop.org/software/systemd/man/sd-daemon.html

use crate::SocketActivationStatus;

/// Check if the process was socket-activated by systemd.
///
/// Reads the LISTEN_FDS and LISTEN_PID environment variables.
/// When LISTEN_FDS > 0 and LISTEN_PID matches our PID, we are in
/// socket activation mode.
pub fn check_socket_activation() -> SocketActivationStatus {
    let listen_fds: u32 = std::env::var("LISTEN_FDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let listen_pid: Option<u32> = std::env::var("LISTEN_PID").ok().and_then(|v| v.parse().ok());

    let our_pid = std::process::id();

    if listen_fds > 0 && listen_pid == Some(our_pid) {
        SocketActivationStatus {
            state: crate::SocketActivationState::Listening,
            activated_socket_count: listen_fds,
            listen_address: std::env::var("LISTEN_NAMES").ok(),
        }
    } else {
        SocketActivationStatus {
            state: crate::SocketActivationState::None,
            activated_socket_count: 0,
            listen_address: None,
        }
    }
}

/// Check if socket activation handoff is complete.
///
/// After the daemon has accepted all handoff sockets and is running
/// normally, it transitions to HandoffComplete.
pub fn handoff_complete(status: &SocketActivationStatus) -> SocketActivationStatus {
    SocketActivationStatus {
        state: crate::SocketActivationState::HandoffComplete,
        activated_socket_count: status.activated_socket_count,
        listen_address: status.listen_address.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_activation_by_default() {
        // Unset the env vars (they shouldn't be set in test environment)
        std::env::remove_var("LISTEN_FDS");
        std::env::remove_var("LISTEN_PID");

        let status = check_socket_activation();
        assert!(matches!(status.state, crate::SocketActivationState::None));
        assert_eq!(status.activated_socket_count, 0);
    }

    #[test]
    fn test_activation_detected() {
        std::env::set_var("LISTEN_FDS", "2");
        std::env::set_var("LISTEN_PID", &std::process::id().to_string());

        let status = check_socket_activation();
        assert!(matches!(status.state, crate::SocketActivationState::Listening));
        assert_eq!(status.activated_socket_count, 2);

        // Cleanup
        std::env::remove_var("LISTEN_FDS");
        std::env::remove_var("LISTEN_PID");
    }

    #[test]
    fn test_handoff_complete() {
        let status = SocketActivationStatus {
            state: crate::SocketActivationState::Listening,
            activated_socket_count: 1,
            listen_address: Some("scmessenger.socket".to_string()),
        };
        let completed = handoff_complete(&status);
        assert!(matches!(completed.state, crate::SocketActivationState::HandoffComplete));
        assert_eq!(completed.activated_socket_count, 1);
    }
}
