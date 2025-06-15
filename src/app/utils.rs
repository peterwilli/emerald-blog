use chrono::DateTime;
use libnostrblog::Timestamp;

pub fn to_date_string(timestamp: Timestamp) -> String {
    // Create NaiveDateTime from timestamp (assuming seconds since epoch)
    if let Some(datetime) = DateTime::from_timestamp(timestamp.as_u64() as i64, 0) {
        // Format as a readable string
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid date".to_owned()
    }
}
