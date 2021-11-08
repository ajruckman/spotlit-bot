use chrono::{DateTime, Utc};
use sqlx::{PgPool, query};
use tokio_stream::StreamExt;

use crate::db::PGExec;
use crate::db::schema::*;

pub async fn add_watch(
    conn: &PgPool,
    id_created_by: u64,
    id_server: u64,
    id_alert_channel: u64,
    id_artist: &str,
    market: &str,
) -> anyhow::Result<Watch> {
    let r = query!(
        "INSERT INTO watch (time_created, id_created_by, id_server, id_alert_channel, id_artist, market, has_initialized, time_last_scanned)
         VALUES (NOW(), $1, $2, $3, $4, $5, FALSE, TO_TIMESTAMP(0))
         ON CONFLICT ON CONSTRAINT watch_server_artist_uniq DO
         UPDATE SET time_created=NOW(), id_created_by=$1, id_server=$2, id_alert_channel=$3, id_artist=$4, market=$5, has_initialized=FALSE, time_last_scanned=TO_TIMESTAMP(0)
         RETURNING id, time_created, time_last_scanned;",
        id_created_by.to_string(), id_server.to_string(), id_alert_channel.to_string(), id_artist, market)
        .fetch_one(conn)
        .await?;

    Ok(Watch {
        id: r.id,
        time_created: r.time_created,
        id_created_by,
        id_server,
        id_alert_channel,
        id_artist: id_artist.to_owned(),
        market: market.to_owned(),
        has_initialized: false,
        time_last_scanned: r.time_last_scanned,
    })
}

pub async fn update_watch(
    conn: &PgPool,
    id: i32,
    has_initialized: bool,
    time_last_scanned: DateTime<Utc>,
) -> anyhow::Result<()> {
    query!(
        "UPDATE watch SET has_initialized=$1, time_last_scanned=$2 WHERE id = $3",
        has_initialized, time_last_scanned, id)
        .execute(conn)
        .await?;

    Ok(())
}

macro_rules! map_watch {
    ($v: expr) => { Watch {
        id: $v.id,
        time_created: $v.time_created,
        id_created_by: $v.id_created_by.parse::< u64 > ().unwrap(),
        id_server: $v.id_server.parse::< u64 > ().unwrap(),
        id_alert_channel: $v.id_alert_channel.parse::< u64 > ().unwrap(),
        id_artist: $v.id_artist,
        market: $v.market,
        has_initialized: $v.has_initialized,
        time_last_scanned: $v.time_last_scanned,
    } }
}

pub async fn list_watches<'a, TDB: PGExec<'a>>(conn: TDB) -> anyhow::Result<Vec<Watch>> {
    let mut stream = query!(
        "SELECT *
         FROM watch"
        )
        .map(|r| map_watch!(r))
        .fetch(conn);

    let mut result = Vec::new();
    while let Some(row) = stream.try_next().await? {
        result.push(row);
    }

    Ok(result)
}

pub async fn list_unalerted_watches<'a, TDB: PGExec<'a>>(conn: TDB) -> anyhow::Result<Vec<PendingWatchAlert>> {
    let mut stream = query!("SELECT * FROM vw_unalerted_watches")
        .map(|r| PendingWatchAlert {
            id_watch: r.id_watch.unwrap(),
            has_initialized: r.has_initialized.unwrap(),
            id_server: r.id_server.unwrap().parse::<u64>().unwrap(),
            id_alert_channel: r.id_alert_channel.unwrap().parse::<u64>().unwrap(),
            market: r.market.unwrap(),
            id_release: r.id_release.unwrap(),
            artist_names: r.artist_names.unwrap(),
            album_type: r.album_type.unwrap(),
            href: r.href.unwrap(),
            image_url: r.image_url.unwrap(),
            name: r.name.unwrap(),
            release_date: r.release_date.unwrap(),
        })
        .fetch(conn);

    let mut result = Vec::new();
    while let Some(row) = stream.try_next().await? {
        result.push(row);
    }

    Ok(result)
}

pub async fn set_watch_alerted(
    conn: &PgPool,
    id_release: &str,
    id_watch: i32,
) -> anyhow::Result<()> {
    query!(
        "INSERT INTO artist_release_watch_alerted (id_release, id_watch)
         VALUES ($1, $2)",
        id_release, id_watch
    )
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn add_artist_release(
    conn: &PgPool,
    id_release: &str,
    id_artist: &str,
    artist_ids: Vec<String>,
    artist_names: Vec<String>,
    album_type: &str,
    available_markets: Vec<String>,
    href: &str,
    image_url: &str,
    name: &str,
    release_date: &str,
    release_date_precision: &str,
) -> anyhow::Result<ArtistRelease> {
    let r = query!(
        "INSERT INTO artist_release (id_release, id_artist, time_first_seen, artist_ids, artist_names, album_type, available_markets, href, image_url, name, release_date, release_date_precision)
         VALUES ($1, $2, NOW(), $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING (time_first_seen)",
        id_release, id_artist, &artist_ids, &artist_names, album_type, &available_markets, href, image_url, name, release_date, release_date_precision)
        .fetch_one(conn)
        .await?;

    Ok(ArtistRelease {
        id_release: id_release.to_owned(),
        id_artist: id_artist.to_owned(),
        time_first_seen: r.time_first_seen,
        artist_ids: artist_ids,
        artist_names: artist_names,
        album_type: album_type.to_owned(),
        available_markets: available_markets,
        href: href.to_owned(),
        image_url: image_url.to_owned(),
        name: name.to_owned(),
        release_date: release_date.to_owned(),
        release_date_precision: release_date_precision.to_owned(),
    })
}

pub async fn list_all_watched_artists<'a, TDB: PGExec<'a>>(conn: TDB) -> anyhow::Result<Vec<String>> {
    let mut stream = query!("SELECT id_artist FROM vw_all_watched_artists")
        .map(|r| r.id_artist.unwrap())
        .fetch(conn);

    let mut result = Vec::new();
    while let Some(row) = stream.try_next().await? {
        result.push(row);
    }

    Ok(result)
}

pub async fn check_known_release(conn: &PgPool, id_release: &str) -> anyhow::Result<bool> {
    let r = query!("SELECT EXISTS(SELECT 1 FROM artist_release WHERE id_release=$1) AS known", id_release)
        .fetch_one(conn)
        .await?;

    Ok(r.known.unwrap())
}

pub async fn check(conn: &PgPool, id_release: &str) -> anyhow::Result<bool> {
    let r = query!("SELECT EXISTS(SELECT 1 FROM artist_release WHERE id_release=$1) AS known", id_release)
        .fetch_one(conn)
        .await?;

    Ok(r.known.unwrap())
}
