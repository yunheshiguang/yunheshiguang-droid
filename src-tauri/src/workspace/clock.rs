use std::time::{SystemTime, UNIX_EPOCH};

use chrono::DateTime;
use yunheshiguang_contract::wkt::Timestamp;

pub fn timestamp(at: SystemTime) -> Timestamp {
    let elapsed = at.duration_since(UNIX_EPOCH).unwrap_or_default();

    Timestamp {
        seconds: i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX),
        nanos: i32::try_from(elapsed.subsec_nanos()).unwrap_or_default(),
        ..Default::default()
    }
}

pub fn now() -> Timestamp {
    timestamp(SystemTime::now())
}

pub fn from_datetime<Tz: chrono::TimeZone>(at: &DateTime<Tz>) -> Timestamp {
    Timestamp {
        seconds: at.timestamp(),
        nanos: i32::try_from(at.timestamp_subsec_nanos()).unwrap_or_default(),
        ..Default::default()
    }
}
