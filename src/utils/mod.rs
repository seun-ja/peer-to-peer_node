pub mod error;

use std::time::{Duration, SystemTime};

pub type UnixTimestamp = i64;

pub fn unix_epoch_time() -> UnixTimestamp {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis() as i64
}
