mod configs;
mod utils;
mod events;
mod tables;
mod commands;

use std::fs;
use std::path::Path;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use serenity::Client;
use serenity::client::bridge::gateway::{GatewayIntents};
use serenity::framework::StandardFramework;
use sqlx::mysql::MySqlPoolOptions;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::configs::ConfigData;
use crate::events::route::Router;
use crate::utils::client::Components;

static LOG_DIR: &str = "logs";
static LOG_FILE : &str = "bot_log";
pub static STATIC_COMPONENTS: Lazy<Mutex<Components>> = Lazy::new(|| Mutex::new(Components::new()));

#[tokio::main]
async fn main() {
    let is_debug = debug_arg_check();

    if let Err(error) = init_logger(is_debug) {
        println!("Init Error: {}", error);
        return;
    }

    if is_debug {
        debug!("Enabled debug mode...");
    }

    let config_path = if is_debug { "./configs/config.yaml" } else { "./config.yaml" };

    let config_string = fs::read_to_string(Path::new(config_path));
    if let Err(error) = config_string {
        error!("Not found config file. Create './config.yaml' file...");
        error!("{:?}", error);
        return;
    }
    let config_string = config_string.unwrap();

    let config = serde_yaml::from_str::<ConfigData>(&config_string);
    if let Err(error) = config {
        error!("Parse error. Please check if the description is correct.");
        error!("{:?}", error);
        return;
    }
    let config = config.unwrap();

    info!("Database Connecting...");

    let mysql_client = MySqlPoolOptions::new()
        .connect(config.get_db_url().as_str()).await;

    if let Err(error) = mysql_client {
        error!("Database connecting error: {:?}", error);
        return;
    }

    let mysql_client = mysql_client.unwrap();

    info!("Bot Starting...");

    let mut client = create_client(&config).await;


    let mut lsc = STATIC_COMPONENTS.lock().await;
    lsc.sets(config, mysql_client, client.shard_manager.clone());
    std::mem::drop(lsc);

    start_signal();

    if let Err(error) = client.start().await {
        error!("Stop Error: {}", error);
    }
}

fn debug_arg_check() -> bool {
    for arg in std::env::args() {
        if match arg.as_str() {
            "--debug" => {
                true
            },
            _ => {
                false
            }
        } {
            return true;
        }
    }

    return false;
}

fn get_log_path(dir_only: bool) -> String {
    if dir_only {
        format!("./{}/", LOG_DIR)
    }
    else {
        format!("./{}/{}_{}.log", LOG_DIR, chrono::Local::now().format("%Y-%m-%d_%H;%M;%S").to_string(), LOG_FILE)
    }
}

fn log_dir_check() -> std::io::Result<()> {
    let dir_path_str = get_log_path(true);
    let dir_path = Path::new(dir_path_str.as_str());

    if dir_path.exists() && dir_path.is_dir() {
        return Ok(());
    }

    fs::create_dir(dir_path)
}

fn init_logger(is_debug: bool) -> Result<(), fern::InitError> {
    if let Err(error) = log_dir_check() {
        return Err(fern::InitError::Io(error));
    }

    let log_path = get_log_path(false);
    let format_cloned_id_debug = is_debug.clone();
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let mut code = "".to_string();
            if format_cloned_id_debug {
                if let Some(filepath) = record.file() {
                    if let Some(codeline) = record.line() {
                        code = format!("file: {}:{}", filepath, codeline);
                    }
                }
            }

            return out.finish(
                format_args!(
                    "{} [{}] [{}]: {} {}",
                    chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message,
                    code
                )
            );
        })
        .level(if is_debug { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .chain(std::io::stdout())
        .chain(fern::log_file(Path::new(log_path.as_str()))?)
        .apply()?;

    Ok(())
}

async fn create_client(config: &ConfigData) -> Client {
    let token = config.get_token();
    let framework = StandardFramework::new();
    Client::builder(token)
        .intents(GatewayIntents::all())
        .framework(framework)
        .event_handler(Router)
        .application_id(*config.get_bot_id())
        .await
        .expect("Erred at client")
}

fn start_signal() -> JoinHandle<()> {
    tokio::spawn(
        async move {
            if let Err(_) = tokio::signal::ctrl_c().await {
                error!("Could not Ctrl+C signal wait");
                return;
            }

            debug!("Ctrl+C Received!");
            exit(false).await;
        }
    )
}

pub async fn exit(at_exit: bool) {
    let lsc = STATIC_COMPONENTS.lock().await;
    let mut locked_shardmanager = lsc.get_sm().lock().await;
    locked_shardmanager.shutdown_all().await;

    info!("Exiting...");

    while locked_shardmanager.shards_instantiated().await.len() != 0 { }
    info!("Bot logged out.");

    lsc.get_sql().close().await;
    info!("Database closed.");

    if at_exit { std::process::exit(0); }
}
