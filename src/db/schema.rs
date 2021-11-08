use chrono::{DateTime, Utc};

pub struct Watch {
    pub id: i32,
    pub time_created: DateTime<Utc>,
    pub id_created_by: u64,
    pub id_server: u64,
    pub id_alert_channel: u64,

    pub id_artist: String,
    pub market: String,

    pub has_initialized: bool,
    pub time_last_scanned: DateTime<Utc>,
}

pub struct ArtistRelease {
    pub id_release: String,
    pub id_artist: String,
    pub time_first_seen: DateTime<Utc>,

    pub artist_ids: Vec<String>,
    pub artist_names: Vec<String>,
    pub album_type: String,
    pub available_markets: Vec<String>,
    pub href: String,
    pub image_url: String,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
}

pub struct PendingWatchAlert {
    pub id_watch: i32,
    pub has_initialized: bool,
    pub id_server: u64,
    pub id_alert_channel: u64,
    pub market: String,

    pub id_release: String,
    pub artist_names: Vec<String>,
    pub album_type: String,
    pub href: String,
    pub image_url: String,
    pub name: String,
    pub release_date: String,
}
