use std::collections::HashMap;

use evlog::meta;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::guild::Guild;
use serenity::model::interactions::{Interaction, InteractionResponseType, InteractionType};

use crate::commands;
use crate::runtime::get_logger;

pub struct BotHandler {}

impl BotHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EventHandler for BotHandler {
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

                let r: anyhow::Result<()> = handler.unwrap()(ctx, interaction).await;
                match r {
                    Ok(()) => {}
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }
}
