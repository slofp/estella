mod command_define;
mod commands;
mod configs;
mod events;
mod utils;
mod voice;
mod chat;

use crate::configs::ConfigData;
use crate::events::route::Router;
use crate::utils::client::Components;
use log::{debug, error, info};
use sea_orm::Database;
use serenity::all::ApplicationId;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use songbird::driver::DecodeMode;
use songbird::SerenityInit;
use voice::text2speak::init_voicevox;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

static LOG_DIR: &str = "logs";
static LOG_FILE: &str = "bot_log";
pub static STATIC_COMPONENTS: LazyLock<Mutex<Components>> = LazyLock::new(|| Mutex::new(Components::new()));

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

	let config_path = if is_debug {
		"./configs/config.yaml"
	} else {
		"./config.yaml"
	};

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

	info!("Voicevox Initialize...");

	init_voicevox().await;

	info!("Database Connecting...");

	let mysql_client = Database::connect(config.get_db_url().as_str()).await;

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
			"--debug" => true,
			_ => false,
		} {
			return true;
		}
	}

	return false;
}

fn get_log_path(dir_only: bool) -> String {
	if dir_only {
		format!("./{}/", LOG_DIR)
	} else {
		format!(
			"./{}/{}_{}.log",
			LOG_DIR,
			chrono::Local::now().format("%Y-%m-%d_%H;%M;%S").to_string(),
			LOG_FILE
		)
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

			return out.finish(format_args!(
				"{} [{}] [{}]: {} {}",
				chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
				record.target(),
				record.level(),
				message,
				code
			));
		})
		.level(
			if is_debug {
				log::LevelFilter::Debug
			} else {
				log::LevelFilter::Info
			},
		)
		.chain(std::io::stdout())
		.chain(fern::log_file(Path::new(log_path.as_str()))?)
		.apply()?;

	Ok(())
}

async fn create_client(config: &ConfigData) -> Client {
	let token = config.get_token();

	// songbird の初期化を行う
	let songbird_config = songbird::Config::default().decode_mode(DecodeMode::Decode);

	Client::builder(token, GatewayIntents::all())
		.event_handler(Router)
		.register_songbird_from_config(songbird_config)
		.application_id(ApplicationId::from(*config.get_bot_id()))
		.await
		.expect("Erred at client")
}

fn start_signal() -> JoinHandle<()> {
	tokio::spawn(async move {
		if let Err(_) = tokio::signal::ctrl_c().await {
			error!("Could not Ctrl+C signal wait");
			return;
		}

		debug!("Ctrl+C Received!");
		exit(false).await;
	})
}

pub async fn exit(at_exit: bool) {
	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_shard_manager = lsc.get_shard_manager();
	locked_shard_manager.shutdown_all().await;

	info!("Exiting...");

	while locked_shard_manager.shards_instantiated().await.len() != 0 {}
	info!("Bot logged out.");

	let res = lsc.get_sql_client().to_owned().close().await;
	if let Err(e) = res {
		error!("Database disconnect error: {:?}", e);
	}
	info!("Database closed.");

	if at_exit {
		std::process::exit(0);
	}
}
