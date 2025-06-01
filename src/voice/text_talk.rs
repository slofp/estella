use std::sync::Arc;

use deepgram::{common::stream_response::StreamResponse, listen::websocket::TranscriptionStream};
use serenity::all::{ChannelId, CreateMessage, Http};
use songbird::Call;
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::{chat::{create_user_message, getchat_responce, param::ResponseData}, STATIC_COMPONENTS};

use super::text2speak::{create_tts_option, vvclient::{TtsAudioOptionBuilder, VoiceVoxTtsClient, VoiceVoxWord}, MODEL_STYLE_ID, VOICE_VOX_CLIENT};

struct TextTalkIntervalTask {
	io_task: JoinHandle<()>,
	cancel_token: CancellationToken,
	interval_task: Option<JoinHandle<()>>,
}

pub(crate) struct TextTalk {
	cancel_token: CancellationToken,
	io_task: JoinHandle<()>,
	interval_task: JoinHandle<()>,
	text_queue: Arc<Mutex<Vec<String>>>,
}

impl TextTalk {
	pub fn cancel(&self) {
		self.cancel_token.cancel();
	}

	pub fn exited(&self) -> bool {
		self.cancel_token.is_cancelled() && self.io_task.is_finished() && self.interval_task.is_finished()
	}

	pub fn is_canceled(&self) -> bool {
		self.cancel_token.is_cancelled()
	}

	pub async fn waiting_inner_task(self) {
		self.io_task.await.unwrap();
		self.interval_task.await.unwrap();
	}

	pub fn make_task(http: Arc<Http>, vc_handler: Arc<Mutex<Call>>, stream: Arc<Mutex<TranscriptionStream>>) -> Self {
		let cancel_token = CancellationToken::new();

		let in_thread_cancel_token = cancel_token.clone();
		let text_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

		let in_thread_text_queue = Arc::clone(&text_queue);
		let io_thread = tokio::spawn(async move {
			loop {
				let mut stream_lock = stream.lock().await;
				tokio::select! {
					_ = in_thread_cancel_token.cancelled() => {
						break;
					}
					Some(result) = stream_lock.next() => {
						if let Ok(result) = result {
							let mut text_queue = in_thread_text_queue.lock().await;
							Self::action(&mut text_queue, result).await;
						} else {
							let err = result.unwrap_err();
							log::error!("err: {err}");
						}
					}
				}
			}
		});

		let in_thread_cancel_token = cancel_token.clone();
		let in_thread_text_queue = Arc::clone(&text_queue);
		let interval_task = tokio::spawn(async move {
			let mut prev_count = 0;
			let mut start_polling = false;
			loop {
				tokio::select! {
					_ = in_thread_cancel_token.cancelled() => {
						break;
					}
					_ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
						prev_count = Self::interval_task(&in_thread_text_queue, prev_count, &mut start_polling, &http, &vc_handler).await;
					}
				}
			}
		});

		TextTalk {
			io_task: io_thread,
			cancel_token,
			interval_task,
			text_queue,
		}
	}

	pub async fn action(text_queue: &mut Vec<String>, result: StreamResponse) {
		if let StreamResponse::TranscriptResponse {
			channel,
			type_field,
			start,
			duration,
			is_final,
			speech_final,
			from_finalize,
			metadata,
			channel_index,
		} = result
		{
			for alt in channel.alternatives {
				log::debug!("text: {:?}", alt);
				text_queue.push(alt.transcript);
			}
		} else {
			log::debug!("other res: {:?}", result);
		}
	}

	async fn interval_task(text_queue: &Arc<Mutex<Vec<String>>>, prev_count: usize, polling: &mut bool, http: &Arc<Http>, vc_handler: &Arc<Mutex<Call>>) -> usize {
		let mut text_queue_lock = text_queue.lock().await;
		let current_count = text_queue_lock.len();
		if current_count == 0 {
			return 0;
		}
		if prev_count < current_count {
			return current_count;
		}

		let text = text_queue_lock.drain(0..).reduce(|acc, v| acc + &v).unwrap_or(String::new());
		std::mem::drop(text_queue_lock);
		let text = text.trim();
		if text.len() == 0 {
			return 0;
		}

		log::info!("{}", text);

		if *polling {
			*polling = false;
			let data = Self::send_ai_answer_get(text).await;

			Self::send_voice(&data.message, vc_handler).await;

			for action in &data.actions {
				if action.name == "send_message_channel" {
					if let Some(serde_json::Value::String(v)) = action.params.get("text") {
						let ch = http.get_channel(ChannelId::new(502084690191187983)).await.unwrap();
						ch.guild().unwrap().send_message(&http, CreateMessage::new().content(v)).await.unwrap();
					}
				}
			}
		} else if text.starts_with("") {
			log::info!("Talk polling!");
			*polling = true;

			Self::send_voice(&"どうしたの？".to_string(), vc_handler).await;
		}

		0
	}

	async fn send_voice(text: &String, vc_handler: &Arc<Mutex<Call>>) {
		let client = VOICE_VOX_CLIENT.read().await;

		let wav = client.tts(
			text.clone(),
			MODEL_STYLE_ID,
			create_tts_option(),
			None
		).await;
		let wav = wav.as_ref();

		let mut handler_lock = vc_handler.lock().await;
		handler_lock.play_input(wav.to_vec().into());
		std::mem::drop(handler_lock);
	}

	async fn send_ai_answer_get(text: &str) -> ResponseData {
		let comp_lock = STATIC_COMPONENTS.lock().await;
		let prev_id = comp_lock.get_prev_id().map(|v| v.clone());
		std::mem::drop(comp_lock);

		let (data, id) = getchat_responce(
			create_user_message(
				text,
				0,
				"",
				&crate::chat::param::Gender::Male,
				&chrono::Local::now()
			),
			prev_id
		).await.unwrap();

		let mut comp_lock = STATIC_COMPONENTS.lock().await;
		comp_lock.set_prev_id(id);
		std::mem::drop(comp_lock);

		data
	}
}
