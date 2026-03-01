use chrono::{DateTime, Local, Utc};

pub fn format_timestamp(time: DateTime<Utc>) -> String {
    // Get the current time in UTC;
    time.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
