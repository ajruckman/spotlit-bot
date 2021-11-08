#![feature(trait_alias)]

use std::env;

use evlog::{LogEventConsolePrinter, Logger};
use rspotify::{ClientCredsSpotify, Credentials};
use serenity::Client;

use crate::db::dbclient::DBClient;
use crate::handler::{BotData, BotHandler};
use crate::runtime::{get_logger, set_logger};
use crate::spotify::SpotifyClient;

pub mod helpers;

mod runtime;
mod handler;

mod commands;
mod db;
mod monitor;
mod spotify;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let appl: u64 = env::var("SPOTLIT_APPL").expect("expected SPOTLIT_APPL").parse().expect("application ID is invalid");
    let token = env::var("SPOTLIT_TOKEN").expect("expected SPOTLIT_TOKEN");
    let db_url = env::var("DATABASE_URL").expect("expected DATABASE_URL");
    let spotify_id = env::var("SPOTIFY_ID").expect("expected SPOTIFY_ID");
    let spotify_secret = env::var("SPOTIFY_SECRET").expect("expected SPOTIFY_SECRET");

    let mut spotify = ClientCredsSpotify::new(Credentials::new(&spotify_id, &spotify_secret));
    spotify.request_token().await.unwrap();

    // let artist = ArtistId::from_id("25uiPmTg16RbhZWAqwLBy5").unwrap();
    // let charli = spotify.artist(&artist).await.unwrap();
    // println!("{}", charli.name);
    // let albums = spotify.artist_albums(&artist, None, None);
    // let recent = albums.take(10).collect::<Vec<ClientResult<SimplifiedAlbum>>>().await;
    //
    // for album in recent {
    //     println!("{}", album.as_ref().unwrap().name);
    //     println!("{}", album.as_ref().unwrap().release_date.as_ref().unwrap());
    // }

    let mut logger = Logger::default();
    logger.register(LogEventConsolePrinter::default());
    set_logger(logger);

    let db_client = DBClient::new(&db_url).await
        .expect("failed to connect to database");

    let spotify_client = SpotifyClient::new(&spotify_id, &spotify_secret).await
        .expect("failed to connect to Spotify");

    let data = handler::BotData::new(db_client, spotify_client).await;

    let mut client = Client::builder(&token)
        .event_handler(BotHandler {})
        .application_id(appl)
        .await
        .unwrap_or_else(|e| {
            get_logger().error_with_err("Client initialization error.", &e, None);
            panic!("{}", e)
        });
    client.data.write().await.insert::<BotData>(data);

    if let Err(e) = client.start_shards(2).await {
        get_logger().error_with_err("Client error.", e, None);
    }
}
