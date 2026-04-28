//! Stub for if-watch - original requires native compilation via cc crate
//! which doesn't work on Android.
//!
//! This is a replacement that provides the same API surface but with
//! stub implementations that return errors.

use futures::Stream;
use std::pin::Pin;

/// A watcher for network interface changes
pub struct Watcher {
    _private: (),
}

/// Configuration for the watcher
pub struct Config {
    _private: (),
}

impl Config {
    pub fn new() -> Self {
        Config { _private: () }
    }
}

impl Watcher {
    /// Create a new watcher - always fails on Android
    pub fn new() -> Result<Self, std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "if-watch not supported on Android",
        ))
    }

    /// Watch for interface changes - always returns error
    pub async fn watch(&mut self) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "if-watch not supported on Android",
        ))
    }

    /// Get the poll handle
    pub fn poll(&mut self) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "if-watch not supported on Android",
        ))
    }
}

impl Stream for Watcher {
    type Item = Result<(), std::io::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(None)
    }
}
