// Store module — message persistence and deduplication

pub mod backend;
pub mod contacts;
pub mod history;
pub mod inbox;
pub mod outbox;
pub mod relay_custody;

pub use contacts::{Contact, ContactManager};
pub use history::{HistoryManager, HistoryStats, MessageDirection, MessageRecord};
pub use inbox::{Inbox, ReceivedMessage};
pub use outbox::{Outbox, QueuedMessage};
pub use relay_custody::{CustodyMessage, CustodyState, CustodyTransition, RelayCustodyStore};
