// Store module — message persistence and deduplication

pub mod backend;
pub mod inbox;
pub mod outbox;

pub use inbox::{Inbox, ReceivedMessage};
pub use outbox::{Outbox, QueuedMessage};
