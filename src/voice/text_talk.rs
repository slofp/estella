use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, Arc, Weak}};

use entity::{GuildConfig, GuildConfigBehavior, UserData, UserDataBehavior};
use futures::StreamExt;
use openai_dive::v1::resources::response::request::ResponseInputItem;
use rand::Rng;
use sea_orm::{EntityTrait, IntoActiveModel, Set};
use serenity::{all::{ChannelId, CreateMessage, GuildId, Http}, async_trait};
use songbird::{model::id::UserId, tracks::TrackHandle, Call, Event, EventContext, EventHandler, TrackEvent};
use tokio::{sync::{mpsc::{self, Receiver, Sender}, Mutex, RwLock}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use sea_orm::ActiveModelTrait;

use crate::{chat::{create_multi_user_message, create_user_message, getchat_responce, param::{calculate_likability_level_from_message_count, ResponseAction, ResponseData}}, utils::atomic::TimeoutAtomicBool, STATIC_COMPONENTS};

use super::text2speak::{create_tts_option, MODEL_STYLE_ID, VOICE_VOX_CLIENT};

// さすがに200人以上超えたらdiscordもおかしくなると思うので現実的な数値
const MAX_USER_SIZE: usize = 200;

const POLLING_RAND: f32 = 0.2f32;

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
	prev_message_id: Mutex<Option<String>>,
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
			prev_message_id: Mutex::new(None),
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

		let execute_state= self.talk_execute_state.load(Ordering::Acquire);
		let none_execute_target = self.target_talk_user.lock().await.is_none();

		let random_polling = tokio::task::spawn_blocking(|| {
			// 確率によるポーリング (全体的な)
			let mut rng = rand::rng();
			let random_state: f32 = rng.random();
			random_state < POLLING_RAND
		}).await.unwrap();

		let is_multi_users = self.user_speaking_state.read().await.len() != 1;
		let mut speaking_count = 0;
		for (user_id, text) in &user_text {
			let user_id = *user_id;
			let text = text.trim();
			if text.len() == 0 {
				continue;
			}

			log::debug!("{} text: {}", user_id, text);

			// もしかしたら、dropされた後に追加された文章後にspeakがfalseになる可能性がある？
			let current_speak = self.user_speaking_state.read().await
				.get(&user_id)
				.map_or(false, |v| v.get());

			// 実行中なのにターゲットが存在していない場合、それは全体の受付ができる状態を表している
			if execute_state && none_execute_target {
				// ここでは、一旦何もしない。for終了後に考える
				// ただし、ステートの判断用に処理はする
				// TODO: ミュートを考慮する必要がある
				if current_speak {
					speaking_count += 1;
				}
			}

			let execute_target = self.target_talk_user.lock().await.map_or(false, |u| u == user_id);
			// 実行中で、聞き取りターゲットじゃない人は無視 (聞いていない想定)
			if execute_state && !execute_target {
				continue;
			}

			// 実行中で、聞き取りターゲットだけど、まだ喋ってる状態 (待ってあげる)
			if (execute_state && execute_target) && current_speak {
				// まだ喋ってるなら、これ以上に追加されることがあるかもしれないので、戻しとく
				let mut lock = self.text_queue.lock().await;
				let mut queue_text = lock.get(&user_id).map(|v| v.clone()).unwrap_or_default();
				queue_text = text.to_owned() + &queue_text;

				lock.insert(user_id, queue_text);
				continue;
			}

			// この時点でもうすでに話したい内容が確定する (それ以上話すことがないのでレス)
			if execute_state && execute_target {
				let data = self.send_ai_answer_get_for_target(text).await;

				log::debug!("{:?}", data);

				let track = self.send_voice(&data.message).await;
				self.execute_actions(&data.actions).await;

				// voiceのawaitをする
				let (_track_waiter, wait) = TrackHandleWaiter::new(track);
				wait.await.unwrap();
				log::debug!("waited!!!");

				self.after_action(&data.actions).await;
				// 空白で終わる場合は、相槌だけで終わっているということなので、全体の受付に戻る
				if data.message == "" {
					let mut user = self.target_talk_user.lock().await;
					*user = None;
				}

				// もし、ターゲットがないなら、受付を再開する
				if self.target_talk_user.lock().await.is_none() {
					self.talk_execute_state.store(false, Ordering::Release);
				}
				continue;
			}

			// 基本的にこれが引き当たることは無いはずだが、一応実行中は開始しない
			if execute_state {
				continue;
			}

			// これ以外の場合、実行していないので受け付ける

			if current_speak {
				// まだ喋ってるなら、これ以上に追加されることがあるかもしれないので、戻しとく
				let mut lock = self.text_queue.lock().await;
				let mut queue_text = lock.get(&user_id).map(|v| v.clone()).unwrap_or_default();
				queue_text = text.to_owned() + &queue_text;

				lock.insert(user_id, queue_text);
				continue;
			}

			// 別に特定の文字で受け付けているわけではない
			// なので、基本的に全部流すことになりそう？
			// 1-1のときは基本的に全部の言葉を流すようにする
			// 1-多のときは特定文字のポーリングかつ、テキストに対して確率的な流し込みを行う
			// TODO:    その時、一定時間何も話していない場合(テキスト化できていない)は、確率的に話題を出す
			// TODO:     -> 全員がミュートだった時、話す意味がないのでどうしよう

			// 1:1じゃなければ、条件判定
			let mut specified_polling: bool = false;
			if is_multi_users {
				// 特定文字のポーリング

				if !(specified_polling || random_polling) {
					continue;
				}
			}

			// 上記条件が通れば、実行状態になる。

			self.talk_execute_state.store(true, Ordering::Release);

			//self.send_voice(&"どうしたの？".to_string()).await;
			// リアルタイム会話に齟齬がないように、戻しとく
			// つまり、ここではステートの管理だけをし、実際の実行は次のターンに回す
			let mut lock = self.text_queue.lock().await;
			let mut queue_text = lock.get(&user_id).map(|v| v.clone()).unwrap_or_default();
			queue_text = text.to_owned() + &queue_text;
			lock.insert(user_id, queue_text);

			// 一人だけ もしくは マルチ時に特定の文字を検知したのであれば、ターゲット
			if !is_multi_users || specified_polling {
				let mut user = self.target_talk_user.lock().await;
				*user = Some(user_id);
			}
		}

		// マルチ系一括実行処理

		// 実行中なのにターゲットが存在していない場合、それは全体の受付ができる状態を表している
		// この実行中ステートは変更前のステートなのでfor内で変更があっても問題はないはずである
		if execute_state && none_execute_target {
			if speaking_count > 1 {
				// 2人以上で喋ってたら待つ
				for (user_id, text) in &user_text {
					let user_id = *user_id;
					let text = text.trim();
					if text.len() == 0 {
						continue;
					}

					let mut lock = self.text_queue.lock().await;
					let mut queue_text = lock.get(&user_id).map(|v| v.clone()).unwrap_or_default();
					queue_text = text.to_owned() + &queue_text;

					lock.insert(user_id, queue_text);
				}
			} else {
				// 誰か一人でも喋ってない人が居るならば、
				// 確実にその人のフォーカスを合うようになるはずであるため
				// 他の人の会話を一旦遮った状態の全インプットデータを使用する

				let user_messages: Vec<_> = futures::stream::iter(user_text.into_iter())
					.filter_map(|(i, t)| async move {
						let t = t.trim();
						if t.len() == 0 {
							None
						} else {
							Some(self.create_single_user_message(t, i).await)
						}
					})
					.collect().await;

				let users_message_item = create_multi_user_message(
					user_messages.iter().map(|(v,_,_)| v).collect()
				);

				// 実行フェーズ

				let data = self.send_ai_answer_get_for_multi_user(user_messages, users_message_item).await;

				log::debug!("{:?}", data);

				let track = self.send_voice(&data.message).await;
				self.execute_actions(&data.actions).await;

				// voiceのawaitをする
				let (_track_waiter, wait) = TrackHandleWaiter::new(track);
				wait.await.unwrap();
				log::debug!("waited!!!");

				self.after_action(&data.actions).await;
				// 空白で終わる場合は、相槌だけで終わっているということなので、全体の受付に戻る
				if data.message == "" {
					let mut user = self.target_talk_user.lock().await;
					*user = None;
				}

				// もし、ターゲットがないなら、受付を再開する
				if self.target_talk_user.lock().await.is_none() {
					self.talk_execute_state.store(false, Ordering::Release);
				}
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

	async fn after_action(&self, actions: &Vec<ResponseAction>) {
		for action in actions {
			if action.name == "after_want_answer" {
				// 質問相手の切り替えに使われる
				if let Some(serde_json::Value::Number(v)) = action.params.get("user_id") {
					let change_user_id = v.as_u64().map(|v| UserId(v));

					let mut user = self.target_talk_user.lock().await;
					*user = change_user_id;
				}
			} else if action.name == "end_topic" {
				// 話題の終了、入力受付に戻る
				let mut user = self.target_talk_user.lock().await;
				*user = None;
				self.talk_execute_state.store(false, Ordering::Release);
			}
		}
	}

	async fn send_voice(&self, text: &String) -> TrackHandle {
		let client = VOICE_VOX_CLIENT.read().await;

		// ～とかーは1こだと短すぎるので、2こに増やしておく
		let text = text.replace("ー", "ーー").replace("～", "～～");

		let wav = client.tts(
			text,
			MODEL_STYLE_ID,
			create_tts_option(),
			None
		).await;
		let wav = wav.as_ref();

		if let Some(vc_handler) = self.vc_handler.upgrade() {
			let mut handler_lock = vc_handler.lock().await;
			let track = handler_lock.play_only_input(wav.to_vec().into());
			std::mem::drop(handler_lock);
			track
		} else {
			unreachable!();
		}
	}

	async fn create_single_user_message(&self, text: &str, user_id: UserId) -> (ResponseInputItem, UserData, u32) {
		let user_data = self.get_user_data(user_id).await;
		if user_data.is_none() {
			log::error!("UserData is not found.");
			unreachable!();
		}
		let user_data = user_data.unwrap();

		// TODO: 無い場合、聞く？
		if user_data.call_name.is_none() || user_data.gender.is_none() || user_data.chat_message_count.is_none() {
			log::error!("user profile is none");
			unimplemented!();
		}

		let user_message_count = user_data.chat_message_count.unwrap_or_default();
		let user_message = create_user_message(
			text,
			calculate_likability_level_from_message_count(user_message_count),
			user_data.uid,
			user_data.call_name.as_ref().map(|c| c.clone()).unwrap(),
			user_data.gender.as_ref().map(|g| g.clone()).unwrap(),
			&chrono::Local::now()
		);

		(user_message, user_data, user_message_count)
	}

	async fn create_target_user_message(&self, text: &str) -> (ResponseInputItem, UserData, u32) {
		let user_id_lock = self.target_talk_user.lock().await;
		let user_id = user_id_lock.unwrap();
		std::mem::drop(user_id_lock);

		self.create_single_user_message(text, user_id).await
	}

	async fn send_ai_answer_get_for_target(&self, text: &str) -> ResponseData {
		let prev_id = self.prev_message_id.lock().await.as_ref().map(|v| v.clone());

		let (user_message, user_data, user_message_count) = self.create_target_user_message(text).await;
		let (data, id) = getchat_responce(
			user_data.uid,
			user_message,
			prev_id
		).await.unwrap();

		// 前回の会話idを保持
		let mut prev_lock = self.prev_message_id.lock().await;
		*prev_lock = Some(id);
		std::mem::drop(prev_lock);

		// ユーザーのメッセージカウントをインクリメント
		let mut user_data = user_data.into_active_model();
		user_data.chat_message_count = Set(Some(user_message_count + 1));
		let lsc = STATIC_COMPONENTS.lock().await;
		let mysql_client = lsc.get_sql_client();
		let _ = user_data.update(mysql_client).await.inspect_err(|e| {
			log::error!("{:?}", e);
		});
		std::mem::drop(lsc);

		data
	}

	async fn send_ai_answer_get_for_multi_user(&self, users: Vec<(ResponseInputItem, UserData, u32)>, message: ResponseInputItem) -> ResponseData {
		let prev_id = self.prev_message_id.lock().await.as_ref().map(|v| v.clone());

		let (data, id) = getchat_responce(
			// 代表者1名
			users.first().unwrap().1.uid,
			message,
			prev_id
		).await.unwrap();

		// 前回の会話idを保持
		let mut prev_lock = self.prev_message_id.lock().await;
		*prev_lock = Some(id);
		std::mem::drop(prev_lock);

		// ユーザーのメッセージカウントをインクリメント
		for (_, user_data, user_message_count) in users {
			let mut user_data = user_data.into_active_model();
			user_data.chat_message_count = Set(Some(user_message_count + 1));
			let lsc = STATIC_COMPONENTS.lock().await;
			let mysql_client = lsc.get_sql_client();
			let _ = user_data.update(mysql_client).await.inspect_err(|e| {
				log::error!("{:?}", e);
			});
			std::mem::drop(lsc);
		}

		data
	}

	async fn get_user_data(&self, user_id: UserId) -> Option<UserData> {
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

#[derive(Clone)]
struct TrackHandleWaiter {
	data: Arc<TrackHandleWaiterData>
}

struct TrackHandleWaiterData {
	track: TrackHandle,
	sender: Sender<()>,
}

impl TrackHandleWaiter {
	fn new(track: TrackHandle) -> (Self, JoinHandle<()>) {
		let (sender, mut receiver) = mpsc::channel(1);

		let waiting_task = tokio::spawn(async move {
			loop {
				tokio::select! {
					Some(_) = receiver.recv() => {
						break;
					}
				}
			}
		});

		let this = Self {
			data: Arc::new(TrackHandleWaiterData {
				track,
				sender,
			})
		};

		this.data.track.add_event(TrackEvent::End.into(), this.clone()).unwrap();
		this.data.track.add_event(TrackEvent::Error.into(), this.clone()).unwrap();

		(this, waiting_task)
	}
}

#[async_trait]
impl EventHandler for TrackHandleWaiter {
	async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
		match ctx {
			EventContext::Track(_) => self.data.sender.send(()).await.unwrap(),
			_ => unimplemented!(),
		};

		None
	}
}
