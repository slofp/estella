use serenity::client::Context;
use serenity::model::channel::Message;

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

	if *message.author.id.as_u64() != 0 {
		return;
	}
	if message.content == "estella.logout" {
		println!("logging out...");
		ctx.shard.shutdown_clean();

		tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
		std::process::exit(0);
	}
}