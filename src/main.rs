use mpris::PlayerFinder;
use anyhow::{Result, Context};
use regex::Regex;
use std::{env, thread, time::{self, SystemTime, UNIX_EPOCH}};
use colored::Colorize;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use dotenv::dotenv;

fn main() -> Result<()> {
    dotenv().ok();
    let discord_client_id = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set in .env");
    let mut client = connect_to_discord(&discord_client_id);
    loop {
        if let Err(e) = find_player(&mut client) {
            if e.to_string().contains("Failed to find active mpris player") {
                println!("{} Failed to find active mpris player", "Error:".red().bold());
                thread::sleep(time::Duration::from_secs(1));
                client.clear_activity().map_err(|e| anyhow::Error::msg(e.to_string()))?;
                continue;
            }
            if e.to_string().contains("Broken pipe") {
                println!("{}", "Discord connection lost. Attempting to reconnect...".red());
                client = connect_to_discord(&discord_client_id);
                continue;
            }
            eprintln!("{} {:?}", "Error:".red().bold(), e);
        }
        thread::sleep(time::Duration::from_secs(15));
    }
}

fn connect_to_discord(discord_client_id:&str) -> DiscordIpcClient {
    loop {
        match DiscordIpcClient::new(&discord_client_id) {
            Ok(mut c) => match c.connect() {
                Ok(_) => return c,
                Err(_) => {
                    println!("{}", "Failed to connect to Discord Client. Retrying in 5 seconds...".red());
                    thread::sleep(time::Duration::from_secs(5));
                }
            },
            Err(_) => {
                println!("{}", "Failed to create IPC client. Retrying in 5 seconds...".red());
                thread::sleep(time::Duration::from_secs(5));
            }
        }
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
    let _artist: &str = metadata.artists().with_context(|| "Failed to find artists array in metadata output, there may be no audio source playing?")?.get(0).with_context(|| "Failed to find artist in metadata output, there may be no audio source playing?")?;
    let mut title = String::from(_title);
    let artist: String = String::from(_artist);
    if re.is_match(_title) {
        title = _title.replace(".flac", "");
    }

    let track_length = metadata.length().with_context(|| "Failed to find length in metadata output")?;
    let track_elapsed = player.get_position().with_context(|| "Failed to get position")?;

    let unix_now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let timestamps = activity::Timestamps::new().start(unix_now as i64 - track_elapsed.as_secs() as i64).end((unix_now as i64 - track_elapsed.as_secs() as i64) + track_length.as_secs() as i64);

    println!("{} Title: {}", "Info:".blue(), title);
    client.set_activity(activity::Activity::new().state(&artist).details(&title).activity_type(activity::ActivityType::Listening).timestamps(timestamps).assets(activity::Assets::new().large_image("untitled"))).map_err(|e| anyhow::Error::msg(e.to_string()))?;
    Ok(())
}
