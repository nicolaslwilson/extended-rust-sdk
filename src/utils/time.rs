use chrono::{DateTime, Utc};

/// Get the current UTC time.
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

/// Convert a UTC datetime to epoch milliseconds (ceiling).
pub fn to_epoch_millis(value: DateTime<Utc>) -> i64 {
    let millis = value.timestamp_millis();
    let micros = value.timestamp_subsec_micros() % 1000;
    if micros > 0 {
        millis + 1
    } else {
        millis
    }
}
