// Store module — message persistence and deduplication

pub mod backend;
pub mod blocked;
pub mod contacts;
pub mod history;
pub mod inbox;
pub mod logs;
pub mod outbox;
pub mod relay_custody;
pub mod storage;
pub mod tracing_init;

pub use backend::StorageBackend;
// Note: BlockedIdentity/BlockedManager exported through blocked_bridge for UniFFI
pub use contacts::{Contact, ContactManager};
pub use history::{HistoryManager, HistoryStats, MessageDirection, MessageRecord};
pub use inbox::{Inbox, ReceivedMessage};
pub use outbox::{Outbox, QueuedMessage};
pub use relay_custody::{
    CustodyEnforcement, CustodyError, CustodyMessage, CustodyState, CustodyTransition,
    RegistrationState, RegistrationStateInfo, RegistrationTransition, RelayCustodyStore,
    RelayRegistry,
};
