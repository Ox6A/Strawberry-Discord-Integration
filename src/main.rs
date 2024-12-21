use mpris::PlayerFinder;
use anyhow::{Result, Context};
use regex::Regex;
use std::{thread, time};
use colored::Colorize;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use hhmmss::Hhmmss;

fn main() -> Result<()> {
    let mut client = DiscordIpcClient::new("").map_err(|e| anyhow::Error::msg(e.to_string()))?; // Add your discord client id here
    client.connect().map_err(|e| anyhow::Error::msg(e.to_string()))?;
    loop {
        if let Err(e) = find_player(&mut client) {
            if e.to_string().contains("Failed to find active mpris player") {
                println!("{} Failed to find active mpris player", "Error:".red().bold());
                thread::sleep(time::Duration::from_secs(1));
                client.clear_activity().map_err(|e| anyhow::Error::msg(e.to_string()))?;
                continue;
            }
            eprintln!("{} {:?}", "Error:".red().bold(), e);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}

fn find_player(client: &mut DiscordIpcClient) -> Result<()> {
    let finder = PlayerFinder::new().with_context(|| "Failed to connect to dbus")?;
    let player = finder.find_active().with_context(|| "Failed to find active mpris player")?;
    let identity = player.identity();
    println!("{} {}", "Player:".green(), identity);
    if identity != "Strawberry" {
        return Ok(());
    }

    let metadata = player.get_metadata().with_context(|| "Failed to get metadata")?;
    let re = Regex::new(r"(.flac)").unwrap();
    let _title: &str = metadata.title().with_context(|| "Failed to find title in metadata output, there may be no audio source playing?")?;
    let mut title = String::from(_title);
    if re.is_match(_title) {
        title = _title.replace(".flac", "");
    }

    let track_length = metadata.length().with_context(|| "Failed to find length in metadata output")?;
    let track_elapsed = player.get_position().with_context(|| "Failed to get position")?;
    let time_remaining_f = format!("{} / {}", time::Duration::from_secs(track_elapsed.as_secs()).hhmmss(), time::Duration::from_secs(track_length.as_secs()).hhmmss());

    println!("{} Title: {}", "Info:".blue(), title);
    client.set_activity(activity::Activity::new().state(&time_remaining_f).details(&title).activity_type(activity::ActivityType::Listening).assets(activity::Assets::new().large_image("untitled"))).map_err(|e| anyhow::Error::msg(e.to_string()))?;
    Ok(())
}