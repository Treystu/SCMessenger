# Ephemeral Messaging & TTL Configuration

The system provides built-in support for ephemeral messaging, allowing messages to be automatically marked for deletion after a specific period has elapsed. This is managed via the Time-To-Live (TTL) logic implemented in the `ephemeral` module.

## How it Works

The expiration logic determines if a message is "expired" by comparing the current system time against the message's creation timestamp plus a predefined TTL duration.

### TtlConfig
The `TtlConfig` struct defines the lifetime of a message:
- `expires_in_seconds`: A `u64` value representing the number of seconds the message should remain valid after creation.

### Expiration Logic
The `is_expired` function performs the following check:
`current_unix_time > (creation_timestamp + expires_in_seconds)`

- If the result is **true**, the message has exceeded its lifetime and should be deleted or ignored.
- If the result is **false**, the message is still within its validity window.

## Usage Example

```rust
use core::message::ephemeral::{TtlConfig, is_expired};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Configure messages to expire after 60 seconds
    let ttl_config = TtlConfig { expires_in_seconds: 60 };

    // Get current timestamp as creation time
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check if the message is expired
    if is_expired(created_at, &ttl_config) {
        println!("Message has expired.");
    } else {
        println!("Message is still valid.");
    }
}
```

## Technical Details
- **Time Source**: The implementation uses `std::time::SystemTime` and `UNIX_EPOCH`.
- **Complexity**: The check is $O(1)$ and relies on standard integer addition and comparison.
- **Edge Cases**: If `expires_in_seconds` is set to `0`, messages are considered expired immediately after their creation timestamp is surpassed by a single second.
