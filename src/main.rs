use std::env;

use evlog::{LogEventConsolePrinter, Logger};
use serenity::Client;

use crate::db::dbclient::DBClient;
use crate::runtime::{get_logger, set_logger};

pub mod helpers;

mod runtime;
mod handler;

mod commands;
mod db;
mod monitor;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let appl: u64 = env::var("SPOTLIT_APPL").expect("expected SPOTLIT_APPL").parse().expect("application ID is invalid");
    let token = env::var("SPOTLIT_TOKEN").expect("expected SPOTLIT_TOKEN");
    let db_url = env::var("DATABASE_URL").expect("expected DATABASE_URL");

    let mut logger = Logger::default();
    logger.register(LogEventConsolePrinter::default());
    set_logger(logger);

    let db_client = DBClient::new(&db_url).await.expect("failed to connect to database");

    let mut client = Client::builder(&token)
        .event_handler(handler::BotHandler::new(db_client))
        .application_id(appl)
        .await
        .unwrap_or_else(|e| {
            get_logger().error_with_err("Client initialization error.", &e, None);
            panic!("{}", e)
        });

    if let Err(e) = client.start_shards(2).await {
        get_logger().error_with_err("Client error.", e, None);
    }
}
