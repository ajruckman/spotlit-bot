use std::sync::Arc;
use chrono::Utc;

use evlog::meta;
use rspotify::clients::BaseClient;
use rspotify::model::{ArtistId, Id};
use serenity::http::Http;
use serenity::model::prelude::ChannelId;
use serenity::utils::Color;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::db;
use crate::db::dbclient::DBClient;
use crate::runtime::get_logger;
use crate::spotify::SpotifyClient;

pub async fn worker(db: Arc<DBClient>, spotify: Arc<SpotifyClient>, http_ref: Arc<Mutex<Option<Arc<Http>>>>) {
    loop {
        let http_opt = http_ref.lock().await;
        match http_opt.as_ref() {
            None => {}
            Some(_) => break,
        }
        drop(http_opt);

        get_logger().debug("Waiting for HTTP client.", None);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    //

    loop {
        let start = Instant::now();

        let artist_ids = match db::model::list_all_watched_artists(db.conn()).await {
            Ok(v) => v,
            Err(e) => {
                get_logger().error("Failed to list watched artists in database.", meta! {
                    "Error" => e,
                });

                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        };

        for artist_id in artist_ids {
            match scan_artist(&db, &spotify, &artist_id).await {
                Ok(()) => {
                    get_logger().info("Scanned artist.", meta! {
                        "ID" => artist_id,
                    });
                }
                Err(e) => {
                    get_logger().error("Failed to update artist.", meta! {
                        "Error" => e,
                        "ArtistID" => artist_id,
                    });
                }
            }
        }

        match process_unalerted_watches(&db, &http_ref).await {
            Ok(_) => {
                get_logger().info("Successfully processed unalerted watches.", None);
            }
            Err(e) => {
                get_logger().error("Failed to process unalerted watches.", meta! {
                    "Error" => e,
                });

                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        }

        //

        let elapsed_secs = start.elapsed().as_secs() as i32;
        let min_sleep = 180 - elapsed_secs;

        if min_sleep > 0 {
            get_logger().debug("Sleeping executor loop.", meta! {
                "ElapsedMS" => start.elapsed().as_millis(),
                "Seconds" => min_sleep,
            });
            tokio::time::sleep(std::time::Duration::from_secs(min_sleep as u64)).await;
        } else {
            get_logger().debug("Executor loop too longer than min duration; re-running immediately.", meta! {
                "ElapsedMS" => start.elapsed().as_millis(),
                "Seconds" => min_sleep,
            });
        }
    }
}

async fn scan_artist(db: &Arc<DBClient>, spotify: &Arc<SpotifyClient>, id: &str) -> anyhow::Result<()> {
    let artist = ArtistId::from_id(id)?;

    let albums = spotify.conn().artist_albums_manual(
        &artist,
        None,
        None,
        Some(20),
        None,
    ).await?;

    for album in albums.items {
        match &album.id {
            None => {
                get_logger().warn("Ignoring release with no release ID.", meta! {
                    "ArtistID" => id,
                    "ReleaseName" => album.name,
                });
            }
            Some(v) => {
                if db::model::check_known_release(db.conn(), v.id()).await? {
                    continue;
                }

                get_logger().warn("Found new release.", meta! {
                    "ArtistID" => id,
                    "ReleaseName" => album.name,
                    "ReleaseID" => v,
                });

                let artist_ids = album.artists.iter()
                    .map(|v| v.id.as_ref().unwrap().id().to_owned()).collect::<Vec<String>>();
                let artist_names = album.artists.iter()
                    .map(|v| v.name.clone()).collect::<Vec<String>>();

                let image_url = album.images.first().unwrap().clone().url;

                db::model::add_artist_release(
                    db.conn(),
                    v.id(),
                    id,
                    artist_ids,
                    artist_names,
                    album.album_type.as_ref().unwrap(),
                    album.available_markets,
                    album.href.as_ref().unwrap(),
                    &image_url,
                    &album.name,
                    album.release_date.as_ref().unwrap(),
                    album.release_date_precision.as_ref().unwrap(),
                ).await?;
            }
        }
    }

    Ok(())
}

async fn process_unalerted_watches(db: &Arc<DBClient>, http_ref: &Arc<Mutex<Option<Arc<Http>>>>) -> anyhow::Result<()> {
    let unalerted_watches = db::model::list_unalerted_watches(db.conn()).await?;

    let http = http_ref.lock().await;
    let http = http.as_ref().unwrap();

    for a in &unalerted_watches {
        if !a.has_initialized {
            db::model::set_watch_alerted(db.conn(), &a.id_release, a.id_watch).await?;
            db::model::update_watch(db.conn(), a.id_watch, true, Utc::now()).await?;
            continue;
        }

        let channel = ChannelId(a.id_alert_channel);

        let r = channel.send_message(http, |c| {
            c.embed(|e| {
                e.author(|a| {
                    a.name("Spotlit");
                    a.icon_url("https://i.imgur.com/sNciPWx.png");

                    a
                });

                e.title(format!("New Spotify release: {}", a.name));
                e.image(a.image_url.clone());
                e.url(a.href.clone());
                e.color(Color::from_rgb(30, 215, 96));
                e.field("Artists", a.artist_names.join(", "), false);
                e.field("Type", match a.album_type.as_ref() {
                    "album" => "Album",
                    "single" => "Single/EP",
                    "compilation" => "Compilation",
                    other => other,
                }, true);
                e.field("Released", a.release_date.clone(), true);
                e
            });

            c
        }).await;

        match r {
            Ok(v) => {
                get_logger().info("Successfully sent watch alert.", meta! {
                    "WatchID" => a.id_watch,
                    "ReleaseID" => a.id_release,
                    "ReleaseName" => a.name,
                    "MessageID" => v.id,
                });

                db::model::set_watch_alerted(db.conn(), &a.id_release, a.id_watch).await?;
                db::model::update_watch(db.conn(), a.id_watch, true, Utc::now()).await?;
            }
            Err(e) => {
                get_logger().error("Failed to send watch alert.", meta! {
                    "WatchID" => a.id_watch,
                    "ReleaseID" => a.id_release,
                    "ReleaseName" => a.name,
                    "Error" => e,
                });
            }
        }
    }

    Ok(())
}
