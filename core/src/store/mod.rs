// Store module — message persistence and deduplication

pub mod backend;
pub mod blocked;
pub mod contacts;
pub mod dedup;
pub mod history;
pub mod inbox;
pub mod ledger_entry;
pub mod logs;
pub mod outbox;
pub mod relay_custody;
pub mod storage;
pub mod sweeper;
pub mod tracing_init;
pub mod transport_memory;

pub use backend::StorageBackend;
// Note: BlockedIdentity/BlockedManager exported through blocked_bridge for UniFFI
pub use contacts::{Contact, ContactManager};
pub use dedup::{DedupAggregateStats, DedupStats, DedupStatsTracker};
pub use history::{HistoryManager, HistoryStats, MessageDirection, MessageRecord};
pub use inbox::{Inbox, ReceivedMessage};
pub use ledger_entry::*;
pub use outbox::{Outbox, QueuedMessage};
pub use relay_custody::{
    CustodyCompatMode, CustodyEnforcement, CustodyError, CustodyMessage, CustodyState,
    CustodyTransition, RegistrationState, RegistrationStateInfo, RegistrationTransition,
    RelayCustodyStore, RelayRegistry,
};
pub use storage::{DiskStats, RetentionConfig, StorageManager};
pub use sweeper::*;
pub use transport_memory::*;
