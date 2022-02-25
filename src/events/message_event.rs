use log::info;
use serenity::client::Context;
use serenity::model::channel::Message;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::Row;
use crate::STATIC_COMPONENTS;

#[derive(sqlx::FromRow)]
struct UserData {
	uid: u64
}

pub async fn execute(ctx: Context, message: Message) {
	println!(
		"user: {}#{:04}\nChannel: {}\nkind: {:?}\ntext: {}\ncreate date: {}",
		message.author.name,
		message.author.discriminator,
		if message.is_private() { "DM".to_string() } else { message.channel_id.name(ctx.cache).await.unwrap_or_else(|| "None".to_string()) },
		message.kind,
		message.content,
		message.timestamp
	);

	if *message.author.id.as_u64() != 0 /*owner userid*/ {
		return;
	}
	if message.content == "estella.logout" {
		println!("logging out...");
		let lsc = STATIC_COMPONENTS.lock().await;
		let mut locked_shardmanager = lsc.get_sm().lock().await;
		locked_shardmanager.shutdown_all().await;

		info!("Exiting...");

		while locked_shardmanager.shards_instantiated().await.len() != 0 { }
		//ctx.shard.shutdown_clean();

		//tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
		std::process::exit(0);
	}
	else if message.content.starts_with("estella.test") {
		println!("estella.test execute!");
		//let t = message.content.replace("estella.test", "").replace(" ", "");
		let lsc = STATIC_COMPONENTS.lock().await;
		let mysql_client = lsc.get_sql();

		let res = sqlx::query("SELECT COUNT(*) as cnt FROM userdata")
			.fetch_one(mysql_client).await;

		if let Ok(res) = res {
			let cnt: i32 = res.get("cnt");
			println!("get value: {}", cnt);
			/*for row in res {
				println!("get value: {}", row.uid);
			}*/
		}
		else if let Err(res) = res {
			println!("{:?}", res);
		}
	}
}