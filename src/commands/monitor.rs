use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType};
use crate::helpers::command_opt;

pub const MONITOR: &str = "monitor";

pub fn monitor_builder(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name(MONITOR)
        .description("Monitor a new Spotify page")
        .create_option(|opt| opt
            .name("artist")
            .description("The link to the artist to monitor (https://open.spotify.com/artist/...)")
            .required(true)
            .kind(ApplicationCommandOptionType::String))
}

pub async fn monitor(ctx: Context, interaction: ApplicationCommandInteraction) -> anyhow::Result<()> {
    let artist = command_opt::find_required(&ctx, &interaction, command_opt::find_string_opt, "artist").await?.unwrap();

    println!("{}", artist);

    Ok(())
}
