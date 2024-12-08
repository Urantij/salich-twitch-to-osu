use crate::twitch::{OsuRelatedTwitchMessage, TwitchWithOsu};
use config::FileFormat;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::mpsc::UnboundedReceiver;

mod map;
mod osu;
mod twitch;

const MY_CONFIG_PATH: &str = "config";

#[derive(Serialize, Deserialize, Debug)]
struct MyConfig {
    twitch_channel_name: String,
    osu: osu::client::OsuConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let settings = read_settings()?;

    let mut osu = osu::client::OsuClient::create_async(settings.osu).await;

    let (mut twitch, map_receiver) = TwitchWithOsu::new(settings.twitch_channel_name);

    let maps_reading_handle = tokio::spawn(async move {
        process_maps(&mut osu, map_receiver).await;
    });

    twitch.start().await.unwrap();
    maps_reading_handle.await.unwrap();

    Ok(())
}

async fn process_maps(
    osu: &mut osu::client::OsuClient,
    mut incoming_messages: UnboundedReceiver<OsuRelatedTwitchMessage>,
) {
    while let Some(msg) = incoming_messages.recv().await {
        if msg.maps.len() == 0 {
            continue;
        }

        // let maps: Vec<_> = msg.maps.iter().map(|map| map.format_to_link()).collect();
        //
        // let maps = maps.join(" ");
        //
        // let message = format!("{}: {maps}", msg.sender);
        let message = format!("{}: \"{}\"", msg.sender, msg.message);

        if let Err(error) = osu.execute_send_message_async(message).await {
            println!("process_maps send error: {}", error);
        }
    }
}

fn read_settings() -> Result<MyConfig, config::ConfigError> {
    config::Config::builder()
        .add_source(
            config::File::with_name(MY_CONFIG_PATH)
                .format(FileFormat::Json)
                .required(true),
        )
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()?
        .try_deserialize::<MyConfig>()
}
