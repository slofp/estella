use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, Arc, Weak}};

use entity::{GuildConfig, GuildConfigBehavior, UserData, UserDataBehavior};
use sea_orm::EntityTrait;
use serenity::all::{ChannelId, CreateMessage, GuildId, Http};
use songbird::{model::id::UserId, Call};
use tokio::{sync::{mpsc::{self, Receiver, Sender}, Mutex, RwLock}, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{chat::{create_user_message, getchat_responce, param::{ResponseAction, ResponseData}}, utils::atomic::TimeoutAtomicBool, STATIC_COMPONENTS};

use super::text2speak::{create_tts_option, MODEL_STYLE_ID, VOICE_VOX_CLIENT};

// さすがに200人以上超えたらdiscordもおかしくなると思うので現実的な数値
const MAX_USER_SIZE: usize = 200;

pub(crate) struct TextTalk {
	cancel_token: CancellationToken,
	// トーク内容を受け取る
	io_task: Arc<Mutex<Option<JoinHandle<()>>>>,
	// 1秒毎の定期実行
	interval_task: Arc<Mutex<Option<JoinHandle<()>>>>,
	text_queue: Arc<Mutex<HashMap<UserId, String>>>,
	// 操作送信用
	http: Arc<Http>,
	// ボイス送信用
	vc_handler: Weak<Mutex<Call>>,
	// グローバルブロードキャストリーダー
	speaking_result_sender: Sender<(UserId, String)>,
	user_speaking_state: Arc<RwLock<HashMap<UserId, TimeoutAtomicBool>>>,
	talk_execute_state: AtomicBool,
	target_talk_user: Mutex<Option<UserId>>,
	target_guild_id: GuildId,
}

impl TextTalk {
	pub(crate) async fn new(target_guild_id: GuildId, http: Arc<Http>, vc_handler: Weak<Mutex<Call>>, user_speaking_state: Arc<RwLock<HashMap<UserId, TimeoutAtomicBool>>>) -> Arc<Self> {
		let (wx, rx) = mpsc::channel::<(UserId, String)>(MAX_USER_SIZE + 1);

		let this = Arc::new(Self {
			cancel_token: CancellationToken::new(),
			io_task: Arc::new(Mutex::new(None)),
			interval_task: Arc::new(Mutex::new(None)),
			text_queue: Arc::new(Mutex::new(HashMap::new())),
			http,
			vc_handler,
			speaking_result_sender: wx,
			user_speaking_state,
			talk_execute_state: AtomicBool::new(false),
			target_talk_user: Mutex::new(None),
			target_guild_id,
		});

		this.make_task(rx).await;

		this
	}

	pub(crate) fn create_sender(&self) -> Sender<(UserId, String)> {
		self.speaking_result_sender.clone()
	}

	pub fn cancel(&self) {
		self.cancel_token.cancel();
	}

	pub async fn exited(&self) -> bool {
		self.cancel_token.is_cancelled() &&
		self.io_task.lock().await.as_ref().map_or(true, |v| v.is_finished()) &&
		self.interval_task.lock().await.as_ref().map_or(true, |v| v.is_finished())
	}

	pub fn is_canceled(&self) -> bool {
		self.cancel_token.is_cancelled()
	}

	pub async fn waiting_inner_task(&mut self) {
		if let Some(v) = self.io_task.lock().await.take() {
			v.await.unwrap();
		}
		if let Some(v) = self.interval_task.lock().await.take() {
			v.await.unwrap();
		}
	}

	pub async fn make_task(self: &Arc<Self>, mut rx: Receiver<(UserId, String)>) {
		let in_thread_self = Arc::clone(&self);
		let io_thread = tokio::spawn(async move {
			loop {
				tokio::select! {
					_ = in_thread_self.cancel_token.cancelled() => {
						break;
					}
					Some((user_id, result)) = rx.recv() => {
						in_thread_self.action(user_id, result).await;
					}
				}
			}
		});

		let in_thread_self = Arc::clone(&self);
		let interval_task = tokio::spawn(async move {
			loop {
				tokio::select! {
					_ = in_thread_self.cancel_token.cancelled() => {
						break;
					}
					_ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
						in_thread_self.interval_task().await;
					}
				}
			}
		});

		let mut lock = self.io_task.lock().await;
		*lock = Some(io_thread);
		let mut lock = self.interval_task.lock().await;
		*lock = Some(interval_task);
	}

	async fn action(&self, user_id: UserId, text: String) {
		let mut lock = self.text_queue.lock().await;
		let mut queue_text = lock.get(&user_id).map(|v| v.clone()).unwrap_or_default();
		queue_text += &text;

		lock.insert(user_id, queue_text);
	}

	async fn interval_task(&self) {
		let mut text_queue_lock = self.text_queue.lock().await;
		let current_count = text_queue_lock.len();
		if current_count == 0 {
			return;
		}

		let user_text: Vec<_> = text_queue_lock.drain().collect();
		std::mem::drop(text_queue_lock);

		for (user_id, text) in user_text {
			let text = text.trim();
			if text.len() == 0 {
				continue;
			}

			log::debug!("{} text: {}", user_id, text);

			let execute_state= self.talk_execute_state.load(Ordering::Acquire);
			let execute_target = self.target_talk_user.lock().await.map_or(false, |u| u == user_id);
			if execute_state && !execute_target {
				continue;
			}

			// もしかしたら、dropされた後に追加された文章後にspeakがfalseになる可能性がある？
			let current_speak = self.user_speaking_state.read().await
				.get(&user_id)
				.map_or(false, |v| v.get());
			if execute_state && execute_target && current_speak {
				// まだ喋ってるなら、これ以上に追加されることがあるかもしれないので、戻しとく
				let mut lock = self.text_queue.lock().await;
				let mut queue_text = lock.get(&user_id).map(|v| v.clone()).unwrap_or_default();
				queue_text = text.to_owned() + &queue_text;

				lock.insert(user_id, queue_text);
				continue;
			}

			// この時点でもうすでに話したい内容が確定する
			if execute_state && execute_target {
				let data = self.send_ai_answer_get(text).await;

				log::debug!("{:?}", data);

				self.send_voice(&data.message).await;
				self.execute_actions(&data.actions).await;

				// 疑問形で終わるのであれば、フォーカスをその人にしておく
				if data.message.ends_with("?") || data.message.ends_with("？") {
					continue;
				}

				let mut user = self.target_talk_user.lock().await;
				*user = None;
				self.talk_execute_state.store(false, Ordering::Release);
				continue;
			}

			// これ以外の場合、実行していないので受け付ける

			if text.starts_with("") {
				self.send_voice(&"どうしたの？".to_string()).await;

				let mut user = self.target_talk_user.lock().await;
				*user = Some(user_id);
				self.talk_execute_state.store(true, Ordering::Release);
				continue;
			}
		}
	}

	async fn execute_actions(&self, actions: &Vec<ResponseAction>) {
		let guild_config = self.get_target_guild_config().await;
		if guild_config.is_none() {
			log::warn!("guild_config is not found.");
			return;
		}
		let guild_config = guild_config.unwrap();

		for action in actions {
			if action.name == "send_message_channel" {
				let ai_chat_channel = guild_config.send_ai_chat_channel_id;
				if ai_chat_channel.is_none() {
					log::warn!("send_ai_chat_channel_id is none.");
					continue;
				}
				let ai_chat_channel = ai_chat_channel.unwrap();

				if let Some(serde_json::Value::String(v)) = action.params.get("text") {
					let ch = self.http.get_channel(ChannelId::new(ai_chat_channel)).await.unwrap();
					ch.guild().unwrap().send_message(&self.http, CreateMessage::new().content(v)).await.unwrap();
				}
			}
		}
	}

	async fn send_voice(&self, text: &String) {
		let client = VOICE_VOX_CLIENT.read().await;

		let wav = client.tts(
			text.clone(),
			MODEL_STYLE_ID,
			create_tts_option(),
			None
		).await;
		let wav = wav.as_ref();

		if let Some(vc_handler) = self.vc_handler.upgrade() {
			let mut handler_lock = vc_handler.lock().await;
			handler_lock.play_input(wav.to_vec().into());
			std::mem::drop(handler_lock);
		}
	}

	async fn send_ai_answer_get(&self, text: &str) -> ResponseData {
		let comp_lock = STATIC_COMPONENTS.lock().await;
		let prev_id = comp_lock.get_prev_id().map(|v| v.clone());
		std::mem::drop(comp_lock);

		let user_data = self.get_target_user_data().await;
		if user_data.is_none() {
			log::error!("UserData is not found.");
			unreachable!();
		}
		let user_data = user_data.unwrap();

		// TODO: 無い場合、聞く？
		if user_data.call_name.is_none() || user_data.gender.is_none() || user_data.likability_level.is_none() {
			log::error!("user profile is none");
			unimplemented!();
		}

		let (data, id) = getchat_responce(
			create_user_message(
				text,
				user_data.likability_level.unwrap(),
				user_data.call_name.unwrap(),
				user_data.gender.unwrap(),
				&chrono::Local::now()
			),
			prev_id
		).await.unwrap();

		let mut comp_lock = STATIC_COMPONENTS.lock().await;
		comp_lock.set_prev_id(id);
		std::mem::drop(comp_lock);

		data
	}

	async fn get_target_user_data(&self) -> Option<UserData> {
		let user_id_lock = self.target_talk_user.lock().await;
		if user_id_lock.is_none() {
			return None;
		}
		let user_id = user_id_lock.unwrap();
		std::mem::drop(user_id_lock);

		let lsc = STATIC_COMPONENTS.lock().await;
		let mysql_client = lsc.get_sql_client();

		let user_data = UserDataBehavior::find_by_id(user_id.0).one(mysql_client).await;
		std::mem::drop(lsc);

		if let Err(error) = user_data {
			log::error!("DB Error: {:?}", error);
			return None;
		} else if let Ok(None) = user_data {
			log::error!("not found user data.");
			return None;
		}

		Some(user_data.unwrap().unwrap())
	}

	async fn get_target_guild_config(&self) -> Option<GuildConfig> {
		let lsc = STATIC_COMPONENTS.lock().await;
		let mysql_client = lsc.get_sql_client();

		let guild_config = GuildConfigBehavior::find_by_id(self.target_guild_id.get()).one(mysql_client).await;
		std::mem::drop(lsc);

		if let Err(error) = guild_config {
			log::error!("DB Error: {:?}", error);
			return None;
		} else if let Ok(None) = guild_config {
			log::error!("not found guild config.");
			return None;
		}

		Some(guild_config.unwrap().unwrap())
	}
}

impl Drop for TextTalk {
	fn drop(&mut self) {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async {
				if !self.is_canceled() {
					self.cancel();
				}
				self.waiting_inner_task().await;
			});
		});
	}
}
