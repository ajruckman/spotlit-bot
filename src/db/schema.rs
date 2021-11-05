use chrono::{DateTime, Utc};

pub struct Watch {
    pub id: i32,
    pub time_created: DateTime<Utc>,
    pub id_created_by: u64,
    pub id_server: u64,
    pub id_alert_channel: u64,

    pub id_artist: String,

    pub time_last_scanned: DateTime<Utc>,
}
