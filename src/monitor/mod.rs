use std::sync::Arc;

use dashmap::DashMap;

use crate::db::schema::Watch;

pub async fn worker(watches: Arc<DashMap<i32, Watch>>) {
    let keys = watches.into_read_only().keys().collect::<Vec<&i32>>();
}
