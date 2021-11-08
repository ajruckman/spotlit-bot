use std::collections::HashMap;
use std::sync::Arc;

use evlog::meta;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::http::Http;
use serenity::model::guild::Guild;
use serenity::model::id::GuildId;
use serenity::model::interactions::{Interaction, InteractionResponseType, InteractionType};
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

use crate::commands;
use crate::db::dbclient::DBClient;
use crate::runtime::get_logger;
use crate::spotify::SpotifyClient;

pub struct BotData {
    pub db_client: Arc<DBClient>,
    pub spotify_client: Arc<SpotifyClient>,
    pub http: Arc<Mutex<Option<Arc<Http>>>>,
}

impl BotData {
    pub async fn new(db_client: DBClient, spotify_client: SpotifyClient) -> Self {
        let db_client = Arc::new(db_client);
        let spotify_client = Arc::new(spotify_client);

        // let watches = crate::db::model::list_watches(db_client.conn()).await.unwrap();
        // let mut watch_map = DashMap::new();
        // for watch in watches {
        //     watch_map.insert(watch.id, watch);
        // }
        // let watch_map = Arc::new(watch_map);

        let http = Arc::new(Mutex::new(None));

        let db_client_ref = db_client.clone();
        let spotify_client_ref = spotify_client.clone();
        // let watch_map_ref = watch_map.clone();
        let http_ref = http.clone();

        tokio::spawn(async move {
            crate::monitor::worker(db_client_ref, spotify_client_ref, http_ref).await;
        });

        Self {
            db_client,
            spotify_client,
            http,
        }
    }
}

impl TypeMapKey for BotData {
    type Value = BotData;
}

pub struct BotHandler {}

#[async_trait]
impl EventHandler for BotHandler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let mut data = ctx.data.write().await;
        let bot_data = data.get_mut::<BotData>().unwrap();

        let mut http_ref = bot_data.http.lock().await;
        let _ = http_ref.insert(ctx.http.clone());
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        get_logger().info("Guild ready.", meta![
            "ID" => guild.id,
            "Name" => guild.name,
        ]);

        let existing_cmds = guild.get_application_commands(&ctx).await.unwrap();

        let existing_map = existing_cmds.iter()
            .map(|v| (v.name.clone(), v))
            .collect::<HashMap<_, _>>();

        for cmd in commands::COMMANDS {
            let whitelisted = match cmd.whitelisted_servers {
                None => true,
                Some(servers) => servers.iter().any(|v| v.as_u64() == guild.id.as_u64()),
            };

            if !whitelisted {
                get_logger().debug("Command is not allowed in this server.", meta! {
                    "GuildID" => guild.id,
                    "GuildName" => guild.name,
                    "Command" => cmd.name
                });
                continue;
            }

            if existing_map.contains_key(cmd.name) {
                if !cmd.re_register {
                    get_logger().debug("Command already registered in this server.", meta! {
                        "GuildID" => guild.id,
                        "GuildName" => guild.name,
                        "Command" => cmd.name
                    });
                    continue;
                }
            }

            let created = guild.create_application_command(&ctx.http, |c| {
                (cmd.builder)(c)
            }).await.unwrap();

            get_logger().debug("Registered command in server.", meta! {
                "GuildID" => guild.id,
                "GuildName" => guild.name,
                "Command" => cmd.name,
                "ID" => created.id
            });
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            let guild = ctx.cache.guild(interaction.guild_id.unwrap()).await.unwrap();

            if interaction.kind == InteractionType::Ping {
                get_logger().info("Interaction ping.", meta! {
                    "GuildID" => guild.id,
                    "GuildName" => guild.name,
                    "InteractionID" => interaction.id
                });

                interaction.create_interaction_response(ctx.http.as_ref(), |r| {
                    r.kind(InteractionResponseType::Pong)
                }).await.unwrap();
            } else if interaction.kind == InteractionType::ApplicationCommand {
                get_logger().info("Interaction ping.", meta! {
                        "GuildID" => guild.id,
                        "GuildName" => guild.name,
                        "InteractionID" => interaction.id,
                        "CommandID" => interaction.data.id,
                        "CommandName" => interaction.data.name
                    });

                let handler = commands::get_handler(&interaction.data.name);
                if handler.is_none() { return; }

                let interaction_id = interaction.id;
                let command_id = interaction.data.id.clone();
                let command_name = interaction.data.name.clone();

                let r: anyhow::Result<()> = handler.unwrap()(ctx, interaction).await;
                match r {
                    Ok(()) => {}
                    Err(e) => {
                        get_logger().error("Error occurred in interaction processor.", meta! {
                            "GuildID" => guild.id,
                            "GuildName" => guild.name,
                            "InteractionID" => interaction_id,
                            "CommandID" => command_id,
                            "CommandName" => command_name,
                            "Error" => e,
                        });
                    }
                }
            }
        }
    }
}
