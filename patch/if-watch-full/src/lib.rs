//! Android stub for if-watch
//! This provides the same API surface as the original if-watch crate but
//! with stub implementations that return errors for Android.

pub use ipnet::{IpNet, Ipv4Net, Ipv6Net};

/// An address change event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IfEvent {
    /// A new local address has been added.
    Up(IpNet),
    /// A local address has been deleted.
    Down(IpNet),
}

/// Stub watcher that always returns errors on Android
#[derive(Debug)]
pub struct Watcher;

impl Watcher {
    pub fn new() -> Result<Self, std::io::Error> {
        #[cfg(target_os = "android")]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "if-watch not supported on Android",
            ))
        }
        #[cfg(not(target_os = "android"))]
        {
            Ok(Watcher)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &IpNet> {
        std::iter::empty()
    }

    pub fn poll_if_event(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<IfEvent, std::io::Error>> {
        std::task::Poll::Ready(Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "if-watch not supported on Android",
        )))
    }
}

impl Default for Watcher {
    fn default() -> Self {
        Watcher
    }
}

#[cfg(feature = "tokio")]
pub mod tokio {
    use super::{IpNet, IfEvent, Watcher};
    use futures::stream::{FusedStream, Stream};
    use std::pin::Pin;

    /// A stub IfWatcher for tokio on Android
    #[derive(Debug)]
    pub struct IfWatcher(Watcher);

    impl IfWatcher {
        pub fn new() -> Result<Self, std::io::Error> {
            #[cfg(target_os = "android")]
            {
                Watcher::new().map(|w| IfWatcher(w))
            }
            #[cfg(not(target_os = "android"))]
            {
                Ok(IfWatcher(Watcher))
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = &IpNet> {
            std::iter::empty()
        }

        pub fn poll_if_event(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<IfEvent, std::io::Error>> {
            self.0.poll_if_event(cx)
        }
    }

    impl Default for IfWatcher {
        fn default() -> Self {
            IfWatcher(Watcher::default())
        }
    }

    impl Stream for IfWatcher {
        type Item = Result<IfEvent, std::io::Error>;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            Pin::into_inner(self).poll_if_event(cx).map(Some)
        }
    }

    impl FusedStream for IfWatcher {
        fn is_terminated(&self) -> bool {
            false
        }
    }
}

#[cfg(feature = "smol")]
pub mod smol {
    use super::{IpNet, IfEvent, Watcher};
    use futures::stream::{FusedStream, Stream};
    use std::pin::Pin;

    /// A stub IfWatcher for smol on Android
    #[derive(Debug)]
    pub struct IfWatcher(Watcher);

    impl IfWatcher {
        pub fn new() -> Result<Self, std::io::Error> {
            #[cfg(target_os = "android")]
            {
                Watcher::new().map(|w| IfWatcher(w))
            }
            #[cfg(not(target_os = "android"))]
            {
                Ok(IfWatcher(Watcher))
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = &IpNet> {
            std::iter::empty()
        }

        pub fn poll_if_event(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<IfEvent, std::io::Error>> {
            self.0.poll_if_event(cx)
        }
    }

    impl Default for IfWatcher {
        fn default() -> Self {
            IfWatcher(Watcher::default())
        }
    }

    impl Stream for IfWatcher {
        type Item = Result<IfEvent, std::io::Error>;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            Pin::into_inner(self).poll_if_event(cx).map(Some)
        }
    }

    impl FusedStream for IfWatcher {
        fn is_terminated(&self) -> bool {
            false
        }
    }
}
