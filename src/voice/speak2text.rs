use std::error::Error;

use bytes::Bytes;
use deepgram::{
	common::options::{Encoding, Endpointing, Keyword, Language, Model, Options},
	listen::websocket::TranscriptionStream,
	Deepgram,
};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;

pub(crate) struct Speak2TextStream<E> {
	core: Speak2Text,
	sender: Sender<Result<Bytes, E>>,
	stream: TranscriptionStream,
}

pub(crate) struct Speak2Text {
	client: Deepgram,
}

impl Speak2Text {
	pub fn new<K>(token: K) -> Self
	where
		K: AsRef<str>, {
		let client = Deepgram::new(token).unwrap();

		Self { client }
	}

	pub async fn init<E>(&self) -> (TranscriptionStream, Sender<Result<Bytes, E>>)
	where
		E: Error + Send + Sync + 'static, {
		let transcription = self.client.transcription();
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
			.endpointing(Endpointing::CustomDurationMs(2))
			.stream(ReceiverStream::new(rx))
			.await
			.unwrap();

		(stream, wx)
	}
}
