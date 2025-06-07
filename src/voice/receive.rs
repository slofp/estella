use std::{collections::HashMap, error::Error, fmt::Display, sync::{Arc, Weak}};

use bytes::{BufMut, BytesMut};
use deepgram::Deepgram;
use serenity::{all::{GuildId, Http}, async_trait};
use songbird::{
	events::context_data::VoiceTick, model::{id::UserId, payload::{ClientDisconnect, Speaking}}, Call, Event, EventContext, EventHandler, Songbird
};
use tokio::sync::{Mutex, RwLock};

use crate::utils::atomic::TimeoutAtomicBool;

use super::{disconnect_voice_channel_from_manager, speak2text::Speak2TextStream, text_talk::TextTalk};

#[derive(Debug)]
struct UnknownError;

impl Error for UnknownError {
	fn description(&self) -> &str {
		"Unknown error"
	}
}

impl Display for UnknownError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Unknown error")
	}
}

pub(crate) struct ReceiverData {
	ssrc2user: HashMap<u32, UserId>,
	client: Deepgram,
	user_stream: HashMap<UserId, Speak2TextStream<UnknownError>>,
	user_speaking_state: Arc<RwLock<HashMap<UserId, TimeoutAtomicBool>>>,
	talking_service: Option<Arc<TextTalk>>,
	vc_handler: Weak<Mutex<Call>>,
	http: Arc<Http>,
}

impl ReceiverData {
	pub async fn start(&mut self, guild_id: GuildId) {
		if let Some(action) = self.talking_service.as_ref() {
			if action.exited().await {
				self.talking_service = None;
			} else {
				return;
			}
		}

		let talking_service = TextTalk::new(guild_id, Arc::clone(&self.http), Weak::clone(&self.vc_handler.clone()), Arc::clone(&self.user_speaking_state)).await;
		self.talking_service = Some(talking_service);
	}

	pub async fn create_stream(&mut self, user: UserId) -> Result<(), String> {
		if self.talking_service.is_none() {
			return Err("Talking service is none.".to_string());
		}

		let sender = self.talking_service.as_ref().unwrap().create_sender();
		let stream = Speak2TextStream::new(&self.client, user, sender).await;
		self.user_stream.insert(user, stream);
		self.user_speaking_state.write().await.insert(user, TimeoutAtomicBool::new());

		Ok(())
	}

	pub async fn remove_stream(&mut self, user: UserId) {
		self.user_speaking_state.write().await.remove(&user);
		self.user_stream.remove(&user);
	}

	pub fn stop(&mut self) {
		self.ssrc2user.drain();
		self.user_stream.drain();
		self.talking_service.take();
	}
}

impl Drop for ReceiverData {
	fn drop(&mut self) {
		self.stop();
	}
}

#[derive(Clone)]
pub(crate) struct Receiver {
	pub(crate) data: Arc<RwLock<ReceiverData>>,
	manager: Weak<Songbird>,
	guild_id: GuildId,
}

impl Receiver {
	pub async fn new(manager: Weak<Songbird>, guild_id: GuildId, http: Arc<Http>, handler: Weak<Mutex<Call>>, token: String) -> Self {
		Self {
			data: Arc::new(RwLock::new(ReceiverData {
				ssrc2user: HashMap::new(),
				client: Deepgram::new(token).unwrap(),
				user_stream: HashMap::new(),
				user_speaking_state: Arc::new(RwLock::new(HashMap::new())),
				talking_service: None,
				vc_handler: handler,
				http,
			})),
			manager,
			guild_id,
		}
	}

	// これはもともとstate更新用なのかもしれない、でもClientConnectが無いのでここで管理するしか無い
	async fn state_update(&self, speaking: &Speaking) {
		if let Some(user) = speaking.user_id {
			let mut data = self.data.write().await;
			if data.ssrc2user.contains_key(&speaking.ssrc) {
				return;
			} else if data.user_stream.contains_key(&user) {
				// もしssrcが変わっている場合がある？　その場合はinsertだけする
				data.ssrc2user.insert(speaking.ssrc, user);
				return;
			}

			data.create_stream(user).await.unwrap();
			data.ssrc2user.insert(speaking.ssrc, user);
		}
	}

	async fn client_disconnect(&self, client_disconnect: &ClientDisconnect) {
		let user_id = client_disconnect.user_id;
		let mut data = self.data.write().await;
		data.remove_stream(user_id).await;
		data.ssrc2user.retain(|_, v| v != &user_id);

		// いなくなったら
		if data.ssrc2user.len() == 0 {
			if let Some(manager) = self.manager.upgrade() {
				disconnect_voice_channel_from_manager(&manager, self.guild_id).await.unwrap();
			}
		}
	}

	async fn voice_tick(&self, voice_tick: &VoiceTick) {
		let speaking_count = voice_tick.speaking.len();
		// speaking の逆が silent、つまり speaking + silentをすると全体のVCの人数と等しくなる
		// let total_connect_vc_count = speaking_count + voice_tick.silent.len();
		if speaking_count == 0 {
			return;
		}

		let lock = self.data.read().await;
		for (ssrc, data) in &voice_tick.speaking {
			let user_id = lock.ssrc2user.get(ssrc);
			if user_id.is_none() {
				continue;
			}
			let user_id = user_id.unwrap();

			// ssrc2userに存在するならば、streamも存在するはずである。
			let stream = lock.user_stream.get(user_id).unwrap();

			let state_lock = lock.user_speaking_state.read().await;
			let state = state_lock.get(user_id).unwrap();
			state.set(true).await;
			std::mem::drop(state_lock);

			// 基本的にこのifは通るはずである。通らないのはDecodeModeを見直す必要がある
			if let Some(decoded_voice_bytes) = data.decoded_voice.as_ref() {
				let mut bytes = BytesMut::with_capacity(decoded_voice_bytes.len() * 2);
				for sample in decoded_voice_bytes {
					bytes.put_i16_le(*sample);
				}

				if let Err(e) = stream.sender().send(Ok(bytes.freeze())).await {
					log::error!("{:?}", e);
				}
			}
		}

		for ssrc in &voice_tick.silent {
			let user_id = lock.ssrc2user.get(ssrc);
			if user_id.is_none() {
				continue;
			}
			let user_id = user_id.unwrap();
			let state_lock = lock.user_speaking_state.read().await;
			let state = state_lock.get(user_id).unwrap();
			state.set(false).await;
			std::mem::drop(state_lock);
		}
	}
}

#[async_trait]
impl EventHandler for Receiver {
	async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
		match ctx {
			// 他の音声ユーザーがどのように音声データを送信しているかを記述する。
			// クライアントは、SSRC/UserIDのマッチングを許可するために、少なくとも1つのそのようなパケットを送信しなければならない。
			EventContext::SpeakingStateUpdate(speaking) => self.state_update(speaking).await,
			// 20ミリ秒ごとに受信されるオーディオパケットの並び替えとデコード。
			EventContext::VoiceTick(voice_tick) => self.voice_tick(voice_tick).await,
			EventContext::ClientDisconnect(client_disconnect) => self.client_disconnect(client_disconnect).await,
			_ => unimplemented!(),
		};

		None
	}
}
