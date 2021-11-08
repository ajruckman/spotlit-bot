use evlog::meta;
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::{CacheHttp, Http};
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType};
use serenity::model::Permissions;
use serenity::utils::MessageBuilder;

use crate::db;
use crate::handler::BotData;
use crate::helpers::{command_opt, command_resp};
use crate::runtime::get_logger;

pub const MONITOR: &str = "monitor";

static MATCH_ARTIST_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r#"https://open.spotify.com/artist/(\w+)"#).unwrap());

pub fn monitor_builder(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name(MONITOR)
        .description("Monitor a new Spotify page")
        .create_option(|opt| opt
            .name("artist")
            .description("The link to the artist to monitor (https://open.spotify.com/artist/...)")
            .required(true)
            .kind(ApplicationCommandOptionType::String))
        .create_option(|opt| opt
            .name("market")
            .description("An ISO Alpha-2 country code to monitor releases in (ex: US, GB, EU)")
            .required(true)
            .kind(ApplicationCommandOptionType::String))
        .create_option(|opt| opt
            .name("channel-id")
            .description("The ID of the channel to send alerts in; enable Developer Mode and right click channel to copy ID")
            .required(true)
            .kind(ApplicationCommandOptionType::String))
}

pub async fn monitor(ctx: Context, interaction: ApplicationCommandInteraction) -> anyhow::Result<()> {
    let artist = command_opt::find_required(&ctx, &interaction, command_opt::find_string_opt, "artist").await?.unwrap();
    let market = command_opt::find_required(&ctx, &interaction, command_opt::find_string_opt, "market").await?.unwrap();
    let channel = command_opt::find_required(&ctx, &interaction, command_opt::find_string_opt, "channel-id").await?.unwrap();

    command_resp::reply_deferred_ack(&ctx, &interaction).await?;

    let market = market.to_uppercase();

    //

    let guild_id = match interaction.guild_id {
        None => {
            get_logger().info("Interaction was not used in a guild.", meta! {
                "InteractionID" => interaction.id,
            });
            command_resp::reply_deferred_result(&ctx, &interaction, "Invalid channel ID; should be a positive number like 743962690627698758.").await.unwrap();
            return Ok(());
        }
        Some(v) => v,
    };

    //

    let member_id = interaction.member.as_ref().unwrap();

    let permissions = match member_id.permissions {
        None => {
            get_logger().info("Could not read interaction invoker's permissions.", meta! {
                "InteractionID" => interaction.id,
                "GuildID" => guild_id,
            });
            command_resp::reply_deferred_result(&ctx, &interaction, "Invalid channel ID; should be a positive number like 743962690627698758.").await.unwrap();
            return Ok(());
        }
        Some(v) => v,
    };

    if !permissions.contains(Permissions::ADMINISTRATOR) {
        get_logger().info("Non-administrator attempted to add watch.", meta! {
            "InteractionID" => interaction.id,
            "GuildID" => guild_id,
        });
        command_resp::reply_deferred_result(&ctx, &interaction, "Only members with the 'Administrator' permission may use /monitor.").await.unwrap();
        return Ok(());
    }

    //

    let channel_id = match channel.parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            get_logger().info("Invalid channel ID passed to /monitor.", meta! {
                "InteractionID" => interaction.id,
                "GuildID" => guild_id,
                "ChannelID" => channel,
                "ArtistURL" => artist,
                "Market" => market,
            });
            command_resp::reply_deferred_result(&ctx, &interaction, "Invalid channel ID; should be a positive number like 743962690627698758.").await.unwrap();
            return Ok(());
        }
    };

    let artist_id = match MATCH_ARTIST_ID.captures(&artist) {
        None => {
            get_logger().info("Invalid artist URL passed to /monitor.", meta! {
                "InteractionID" => interaction.id,
                "GuildID" => guild_id,
                "ChannelID" => channel_id,
                "ArtistURL" => artist,
                "Market" => market,
            });
            command_resp::reply_deferred_result(&ctx, &interaction, "Invalid artist link; should be like: `https://open.spotify.com/artist/...`").await.unwrap();
            return Ok(());
        }
        Some(v) => {
            if v.len() != 2 {
                get_logger().info("Invalid artist URL passed to /monitor.", meta! {
                    "InteractionID" => interaction.id,
                    "GuildID" => guild_id,
                    "ChannelID" => channel_id,
                    "ArtistURL" => artist,
                    "Market" => market,
                });
                command_resp::reply_deferred_result(&ctx, &interaction, "Invalid artist link; should be like: `https://open.spotify.com/artist/...`").await.unwrap();
                return Ok(());
            }

            v.get(1).unwrap().as_str()
        }
    };

    //

    let can_send = try_send_message(ctx.http(), guild_id.as_u64(), &channel_id).await;

    if !can_send {
        get_logger().info("Failed to send test message in the target channel.", meta! {
            "InteractionID" => interaction.id,
            "GuildID" => guild_id,
            "ChannelID" => channel_id,
            "ArtistURL" => artist,
            "Market" => market,
        });
        command_resp::reply_deferred_result(&ctx, &interaction, format!(
            "Failed to send a test message in the channel with ID `{}`; do I have permissions to send messages there? Is that channel in this server?",
            channel_id
        )).await.unwrap();
        return Ok(());
    }

    //

    let data = ctx.data.read().await;
    let data = data.get::<BotData>().unwrap();

    let watch = match db::model::add_watch(
        data.db_client.conn(),
        *member_id.user.id.as_u64(),
        *guild_id.as_u64(),
        channel_id,
        artist_id,
        &market,
    ).await {
        Ok(v) => v,
        Err(e) => {
            get_logger().error("Failed to save new artist watch.", meta! {
                "InteractionID" => interaction.id,
                "GuildID" => guild_id,
                "ChannelID" => channel_id,
                "ArtistURL" => artist,
                "Market" => market,
                "Error" => e,
            });

            command_resp::reply_deferred_result(&ctx, &interaction, "Failed to save new artist watch.").await.unwrap();
            return Ok(());
        }
    };

    get_logger().info("Saved new artist watch.", meta! {
        "InteractionID" => interaction.id,
        "GuildID" => guild_id,
        "ChannelID" => channel_id,
        "ArtistURL" => artist,
        "Market" => market,
        "WatchID" => watch.id,
    });

    //

    interaction.create_followup_message(ctx.http, |r| r.create_embed(|e| {
        e.author(|a| {
            a.name("Spotlit");
            a.icon_url("https://i.imgur.com/sNciPWx.png");

            a
        });

        e.title("New release watch created");

        e.field("Artist ID", artist_id, true);
        e.field("Market", market, true);
        e.field("Channel", MessageBuilder::new().channel(channel_id).build(), true);
        e.field("ID", watch.id, true);

        e
    })).await.unwrap();

    Ok(())
}

async fn try_send_message(http: &Http, server_id: &u64, channel_id: &u64) -> bool {
    let channel = match http.get_channel(*channel_id).await {
        Ok(v) => v,
        Err(_) => return false,
    };

    match channel.clone().guild() {
        None => return false,
        Some(v) => if v.guild_id.as_u64() != server_id {
            return false;
        }
    };

    let channel_id = channel.clone().id();

    match channel_id.send_message(http, |f| f.content("Test message; please delete.")).await {
        Ok(v) => {
            let _ = v.delete(http).await;
            true
        }
        Err(_) => false,
    }
}
