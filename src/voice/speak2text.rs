use std::error::Error;

use bytes::Bytes;
use deepgram::{
	common::{options::{Encoding, Endpointing, Keyword, Language, Model, Options}, stream_response::StreamResponse},
	listen::websocket::TranscriptionStream,
	Deepgram,
};
use log::info;
use songbird::model::id::UserId;
use tokio::{sync::mpsc::{self, Sender}, task::JoinHandle};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;

pub(crate) struct Speak2TextStream<E> where E: Error + Send + Sync + 'static {
	sender: Sender<Result<Bytes, E>>,
	user_id: UserId,
	cancel_token: CancellationToken,
	convert_task: Option<JoinHandle<()>>,
}

impl<E> Speak2TextStream<E> where E: Error + Send + Sync + 'static {
	pub(crate) async fn new(client: &Deepgram, user_id: UserId, to_talk_sender: Sender<(UserId, String)>) -> Self {
		let transcription = client.transcription();
		let options = Options::builder()
			.model(Model::Nova2)
			.language(Language::ja)
			.punctuate(true)
			.smart_format(true)
			.keywords_with_intensifiers([])
			.build();

		let (wx, rx) = mpsc::channel(1);

		let stream = transcription
			.stream_request_with_options(options)
			.encoding(Encoding::Linear16)
			.sample_rate(48000)
			.channels(2)
			.keep_alive()
			.no_delay(true)
			.endpointing(Endpointing::CustomDurationMs(40))
			.stream(ReceiverStream::new(rx))
			.await
			.unwrap();

		info!("Deepgram Request ID: {}", stream.request_id());

		let mut this = Self {
			sender: wx,
			user_id,
			cancel_token: CancellationToken::new(),
			convert_task: None,
		};

		this.start_stream(stream, to_talk_sender).await;

		this
	}

	pub(crate) fn sender(&self) -> &Sender<Result<Bytes, E>> {
		&self.sender
	}

	async fn start_stream(&mut self, mut stream: TranscriptionStream, wx: Sender<(UserId, String)>) {
		if self.convert_task.is_some() {
			if self.exited().await {
				self.convert_task = None;
			} else {
				return;
			}
		}

		let task_cancel = self.cancel_token.clone();
		let user_id = self.user_id;
		self.convert_task = Some(tokio::spawn(async move {
			loop {
				tokio::select! {
					_ = task_cancel.cancelled() => {
						break;
					}
					Some(result) = stream.next() => {
						if let Ok(result) = result {
							if let Err(e) = wx.send((user_id, Self::convert_responce(result))).await {
								log::error!("{:?}", e);
							}
						} else {
							let err = result.unwrap_err();
							log::error!("err: {err}");
						}
					}
				}
			}
		}));
	}

	fn convert_responce(res: StreamResponse) -> String {
		log::debug!("speak responce data: {:?}", res);

		if let StreamResponse::TranscriptResponse {
			channel,
			type_field: _,
			start: _,
			duration: _,
			is_final: _,
			speech_final: _,
			from_finalize: _,
			metadata: _,
			channel_index: _,
		} = res {
			let mut text = String::new();
			for alt in channel.alternatives {
				text += &alt.transcript;
			}

			text.split_ascii_whitespace().collect::<Vec<_>>().join("")
		} else {
			log::debug!("other res: {:?}", res);
			String::new()
		}
	}

	pub fn cancel(&self) {
		self.cancel_token.cancel();
	}

	pub async fn exited(&self) -> bool {
		self.cancel_token.is_cancelled() &&
		self.convert_task.as_ref().map_or(true, |v| v.is_finished())
	}

	pub fn is_canceled(&self) -> bool {
		self.cancel_token.is_cancelled()
	}

	pub async fn waiting_inner_task(&mut self) {
		if let Some(v) = self.convert_task.take() {
			v.await.unwrap();
		}
	}
}

impl<E> Drop for Speak2TextStream<E> where E: Error + Send + Sync + 'static {
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
