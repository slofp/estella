use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use bytes::{BufMut, Bytes, BytesMut};
use deepgram::listen::websocket::TranscriptionStream;
use serenity::{all::Http, async_trait};
use songbird::{
	events::context_data::VoiceTick, model::{id::UserId, payload::Speaking}, Call, Event, EventContext, EventHandler
};
use tokio::sync::{mpsc::Sender, Mutex, RwLock};

use super::{speak2text::Speak2Text, text_talk::TextTalk};

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
	speak2text: Speak2Text,
	stream: Arc<Mutex<TranscriptionStream>>,
	send_bytes: Sender<Result<Bytes, UnknownError>>,
	action: Option<TextTalk>,
	vc_handler: Arc<Mutex<Call>>,
	http: Arc<Http>,
}

impl ReceiverData {
	pub fn start(&mut self) {
		if let Some(action) = self.action.as_ref() {
			if action.exited() {
				self.action = None;
			} else {
				return;
			}
		}

		self.action = Some(TextTalk::make_task(Arc::clone(&self.http), Arc::clone(&self.vc_handler), Arc::clone(&self.stream)));
	}

	pub async fn stop(&mut self) {
		let action = self.action.take();
		if let Some(action) = action {
			if !action.is_canceled() {
				action.cancel();
			}

			action.waiting_inner_task().await;
		}
	}
}

impl Drop for ReceiverData {
	fn drop(&mut self) {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async {
				self.stop().await;
			});
		});
	}
}

#[derive(Clone)]
pub(crate) struct Receiver {
	pub(crate) data: Arc<RwLock<ReceiverData>>,
}

impl Receiver {
	pub async fn new(http: Arc<Http>, handler: Arc<Mutex<Call>>, token: String) -> Self {
		let speak2text = Speak2Text::new(token);
		let (stream, send_bytes) = speak2text.init::<UnknownError>().await;

		println!("Deepgram Request ID: {}", stream.request_id());

		Self {
			data: Arc::new(RwLock::new(ReceiverData {
				ssrc2user: HashMap::new(),
				speak2text,
				stream: Arc::new(Mutex::new(stream)),
				send_bytes,
				action: None,
				vc_handler: handler,
				http,
			})),
		}
	}

	async fn state_update(&self, speaking: &Speaking) {
		if let Some(user) = speaking.user_id {
			let mut data = self.data.write().await;
			data.ssrc2user.insert(speaking.ssrc, user);
		}
	}

	async fn voice_tick(&self, voice_tick: &VoiceTick) {
		let speaking_count = voice_tick.speaking.len();
		// speaking の逆が silent、つまり speaking + silentをすると全体のVCの人数と等しくなる
		let total_connect_vc_count = speaking_count + voice_tick.silent.len();

		if speaking_count != 0 {
			let lock = self.data.read().await;
			for (ssrc, data) in &voice_tick.speaking {
				let user_id = lock.ssrc2user.get(ssrc);
				if let Some(user_id) = user_id.as_ref() {
					if user_id.0 != 309999444927971328 {
						continue;
					}
				} else {
					continue;
				}

				// 基本的にこのifは通るはずである。通らないのはDecodeModeを見直す必要がある
				if let Some(decoded_voice_bytes) = data.decoded_voice.as_ref() {
					let mut bytes = BytesMut::with_capacity(decoded_voice_bytes.len() * 2);
					for sample in decoded_voice_bytes {
						bytes.put_i16_le(*sample);
					}

					if let Err(e) = self.data.read().await.send_bytes.send(Ok(bytes.freeze())).await {
						log::error!("{:?}", e);
					}
				}
			}
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
			_ => unimplemented!(),
		};

		None
	}
}
