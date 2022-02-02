mod configs;
mod utils;

use std::fs;
use std::path::Path;
use serenity::Client;
use serenity::framework::StandardFramework;
use crate::configs::ConfigData;

fn main() {
    let mut is_debug = false;
    for arg in std::env::args() {
        match arg.as_str() {
            "--debug" => {
                is_debug = true;
            },
            _ => {}
        }
    }

    let config_path = if is_debug { "../../configs/config.yaml" } else { "./config.yaml" };

    let config_string = fs::read_to_string(Path::new(config_path))?;
    let config = serde_yaml::from_str::<ConfigData>(&config_string)?;

    println!("Starting...");

    let framework = StandardFramework::new();

    let token = config.get_token();
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Erred at client");

    if let Err(error) = client.start().await {
        println!("Stop Error: {}", error);
    }
}
