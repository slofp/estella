use std::sync::LazyLock;

use tokio::sync::RwLock;
use vvclient::{TtsAudioOption, TtsAudioOptionBuilder, VoiceVoxTtsClient, VoiceVoxWord, WavData};

pub(crate) mod vvclient;

const JTALK_DIR_PATH: &'static str = "voicevox_core/dict/open_jtalk_dic_utf_8-1.11";
const RUNTIME_PATH: &'static str = "voicevox_core/onnxruntime/lib/voicevox_onnxruntime.dll";
const MODEL_PATH: &'static str = "voicevox_core/models/vvms/.vvm";
pub(crate) const MODEL_STYLE_ID: u32 = 0;

pub(super) static VOICE_VOX_CLIENT: LazyLock<RwLock<VoiceVoxTtsClient>> = LazyLock::new(|| {
	RwLock::new(VoiceVoxTtsClient::new(JTALK_DIR_PATH, Some(RUNTIME_PATH), None))
});

pub(crate) async fn init_voicevox() {
	let mut client = VOICE_VOX_CLIENT.write().await;
	client.load_model(MODEL_PATH).await;
}

pub(crate) fn create_tts_option() -> Option<TtsAudioOption> {
	Some(
		TtsAudioOptionBuilder::new()
			.output_sampling_rate(48000)
			.output_stereo(true)
			.build()
	)
}
