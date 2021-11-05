use sqlx::{PgPool, query};

use tokio_stream::StreamExt;

use crate::db::schema::*;

pub async fn add_watch(
    conn: &PgPool,
    id_created_by: u64,
    id_server: u64,
    id_alert_channel: u64,
    id_artist: &str,
) -> anyhow::Result<Watch> {
    let r = query!("INSERT INTO watch (time_created, id_created_by, id_server, id_alert_channel, id_artist, time_last_scanned)
                    VALUES (NOW(), $1, $2, $3, $4, TO_TIMESTAMP(0))
                    RETURNING id, time_created, time_last_scanned;",
        id_created_by.to_string(), id_server.to_string(), id_alert_channel.to_string(), id_artist)
        .fetch_one(conn)
        .await?;

    Ok(Watch {
        id: r.id,
        time_created: r.time_created,
        id_created_by,
        id_server,
        id_alert_channel,
        id_artist: id_artist.to_owned(),
        time_last_scanned: r.time_last_scanned,
    })
}

pub async fn list_watches(
    conn: &PgPool,
) -> anyhow::Result<Vec<Watch>> {
    let mut stream = query!("SELECT id, time_created, id_created_by, id_server, id_alert_channel, id_artist, time_last_scanned FROM watch")
        .map(|r| Watch {
            id: r.id,
            time_created: r.time_created,
            id_created_by: r.id_created_by.parse::<u64>().unwrap(),
            id_server: r.id_server.parse::<u64>().unwrap(),
            id_alert_channel: r.id_alert_channel.parse::<u64>().unwrap(),
            id_artist: r.id_artist,
            time_last_scanned: r.time_last_scanned,
        })
        .fetch(conn);

    let mut result = Vec::new();
    while let Some(row) = stream.try_next().await? {
        result.push(row);
    }

    Ok(result)
}
