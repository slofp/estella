#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, CStr, CString};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

include!("./voicevox_core.rs");

struct VoiceVoxModel {
	// safety:
	// 参照先のメモリは有効である (ただしメモリアラインメントに沿っている必要は無い)。
	// 参照先のメモリは他スレッドからアクセス中ではない。
	model: *mut VoicevoxVoiceModelFile,
}

impl VoiceVoxModel {
	pub(crate) fn new<P: Into<Vec<u8>>>(file_path: P) -> Self {
		unsafe {
			let mut model = std::ptr::null_mut::<VoicevoxVoiceModelFile>();
			let path = CString::new(file_path).unwrap();
			voicevox_voice_model_file_open(path.as_ptr(), &mut model);

			VoiceVoxModel {
				model,
			}
		}
	}
}

impl Drop for VoiceVoxModel {
	fn drop(&mut self) {
		unsafe { voicevox_voice_model_file_delete(self.model) };
	}
}

// リザルト処理をしてないのでしないといけない
pub(crate) struct VoiceVoxTtsClient {
	// safety:
	// 参照先のメモリは有効である (ただしメモリアラインメントに沿っている必要は無い)。
	// 参照先のメモリは他スレッドからアクセス中ではない。
	synthesizer: *mut VoicevoxSynthesizer,
	jtalk: *mut OpenJtalkRc,
	dict: Option<*mut VoicevoxUserDict>,
	current_model: Option<VoiceVoxModel>,
	sem: Semaphore,
}

// safety:
// synthesizer/jtalk/current_modelにアクセスをしうる関数には必ずその時点でセマフォを取得しなければならない。
unsafe impl Send for VoiceVoxTtsClient {}
unsafe impl Sync for VoiceVoxTtsClient {}

impl VoiceVoxTtsClient {
	pub(crate) fn new<R: Into<Vec<u8>>, J: Into<Vec<u8>>>(
		jtalk_dir_path: J,
		runtime_path: Option<R>,
		init_option: Option<VoicevoxInitializeOptions>
	) -> Self {
		unsafe {
			let mut runtime_option = voicevox_make_default_load_onnxruntime_options();
			// ifでas_ptrすると、CStringはifで消滅するのでエラーになる
			let runtime_path = runtime_path.map(|v| CString::new(v).unwrap());
			if let Some(runtime_path) = runtime_path.as_ref() {
				runtime_option.filename = runtime_path.as_ptr();
			}

			let mut runtime = std::ptr::null::<VoicevoxOnnxruntime>();
			voicevox_onnxruntime_load_once(runtime_option, &mut runtime);

			let dir_name = CString::new(jtalk_dir_path).unwrap();
			let mut jtalk = std::ptr::null_mut::<OpenJtalkRc>();
			voicevox_open_jtalk_rc_new(dir_name.as_ptr(), &mut jtalk);

			let options = match init_option {
				Some(v) => v,
				None => voicevox_make_default_initialize_options(),
			};

			let mut synthesizer = std::ptr::null_mut::<VoicevoxSynthesizer>();
			voicevox_synthesizer_new(runtime, jtalk, options, &mut synthesizer);

			Self {
				synthesizer,
				jtalk,
				dict: None,
				current_model: None,
				sem: Semaphore::new(1),
			}
		}
	}

	pub(crate) async fn load_model<P: Into<Vec<u8>>>(&mut self, file_path: P) {
		// もし読み込み済みのモデルが存在した場合、そのモデルを破棄しなければならないが、
		// 仮にsynthesizerが使用されていた場合、モデルの読み込みが処理の合間に起こってしまう
		// ので、ロックする
		let perm = self.sem.acquire().await.unwrap();

		if let Some(m) = self.current_model.take() {
			std::mem::drop(m);
		}

		let model = VoiceVoxModel::new(file_path);
		unsafe { voicevox_synthesizer_load_voice_model(self.synthesizer, model.model); }

		self.current_model = Some(model);

		std::mem::drop(perm);
	}

	pub(crate) async fn set_dict(&mut self, words: Vec<VoiceVoxWord>) {
		let dict = match self.dict {
			Some(v) => v,
			None => {
				let d = unsafe { voicevox_user_dict_new() };
				self.dict = Some(d);
				d
			},
		};

		for word in words {
			let surface = CString::new(word.0).unwrap();
			let pronunciation = CString::new(word.1).unwrap();
			let w = unsafe { voicevox_user_dict_word_make(surface.as_ptr(), pronunciation.as_ptr(), word.2) };

			let mut id: [u8; 16] = [0; 16];
			// 各引数はこのforループスコープ内でのみ有効であるため問題はない
			unsafe { voicevox_user_dict_add_word(dict, &w, &mut id) };
		}

		// この時点でjtalkのアクセスが必要であるためセマフォが必要
		let perm = self.sem.acquire().await.unwrap();
		unsafe { voicevox_open_jtalk_rc_use_user_dict(self.jtalk, dict); }
		std::mem::drop(perm);
	}

	pub(crate) async fn tts<T: Into<Vec<u8>>>(&self, text: T, style_id: VoicevoxStyleId, option: Option<TtsAudioOption>, synthesis_options: Option<VoicevoxSynthesisOptions>) -> WavData {
		let textc = CString::new(text).unwrap();

		// synthesizerアクセス
		let perm = self.sem.acquire().await.unwrap();
		// selfはmove出来ないのでspawn_blockingは使用できない
		let mut audio_query = tokio::task::block_in_place(move || {
			let mut json_ptr = std::ptr::null_mut::<c_char>();
			unsafe { voicevox_synthesizer_create_audio_query(self.synthesizer, textc.as_ptr(), style_id, &mut json_ptr) };
			std::mem::drop(perm);

			// CString::from_rawのように、Rust側が開放権をもつような動作にしてはならない(アロケータ破損)
			let json_c = unsafe { CStr::from_ptr(json_ptr) };
			let text_json = json_c.to_str().unwrap();
			let audio_query: AudioQuery = serde_json::from_str(text_json).unwrap();
			// safety:
			// json_ptrはこの関数内のみで処理/消費されるポインタであるため問題はない
			unsafe { voicevox_json_free(json_ptr) };

			audio_query
		});

		// NOTE: 事前に調整したパラメータ
		if let Some(option) = option {
			if let Some(v) = option.speed_scale {
				audio_query.speed_scale = v;
			}
			if let Some(v) = option.pitch_scale {
				audio_query.pitch_scale = v;
			}
			if let Some(v) = option.intonation_scale {
				audio_query.intonation_scale = v;
			}
			if let Some(v) = option.volume_scale {
				audio_query.volume_scale = v;
			}
			if let Some(v) = option.pre_phoneme_length {
				audio_query.pre_phoneme_length = v;
			}
			if let Some(v) = option.post_phoneme_length {
				audio_query.post_phoneme_length = v;
			}

			if let Some(v) = option.pause_length_scale {
				for phrase in &mut audio_query.accent_phrases {
					if let Some(mora) = &mut phrase.pause_mora {
						// 変わる？ でも一応pauが固定ぽそう
						if mora.vowel == "pau" {
							mora.vowel_length *= v;
						}
					}
				}
			}

			if let Some(v) = option.output_sampling_rate {
				audio_query.output_sampling_rate = v;
			}

			if let Some(v) = option.output_stereo {
				audio_query.output_stereo = v;
			}
		}

		let options = match synthesis_options {
			Some(v) => v,
			None => unsafe { voicevox_make_default_synthesis_options() },
		};

		let fixed_json_text = serde_json::to_string(&audio_query).unwrap();
		let fixed_json_text_cstr = CString::new(fixed_json_text.as_str()).unwrap();

		// synthesizerアクセス
		let perm = self.sem.acquire().await.unwrap();
		// selfはmove出来ないのでspawn_blockingは使用できない
		tokio::task::block_in_place(move || {
			let mut length = 0usize;
			let mut data = std::ptr::null_mut::<u8>();

			unsafe {
				voicevox_synthesizer_synthesis(
					self.synthesizer,
					fixed_json_text_cstr.as_ptr(),
					style_id,
					options,
					&mut length,
					&mut data
				);
			}
			std::mem::drop(perm);

			WavData::new(length, data)
		})
	}
}

impl Drop for VoiceVoxTtsClient {
	fn drop(&mut self) {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async {
				// permが回復するまでは開放してはならない。
				let perm = self.sem.acquire().await.unwrap();

				unsafe {
					if let Some(dict) = self.dict {
						voicevox_user_dict_delete(dict);
					}
					voicevox_open_jtalk_rc_delete(self.jtalk);
					voicevox_synthesizer_delete(self.synthesizer);
				};

				// 解放後、アクセスしてはいけないのでcloseする
				self.sem.close();
				std::mem::drop(perm);
			});
		});
	}
}

pub(crate) struct VoiceVoxWord(String, String, usize);

impl VoiceVoxWord {
	pub(crate) fn new<S: Into<String>, P: Into<String>>(surface: S, pronunciation: P, accent_type: usize) -> Self {
		Self(surface.into(), pronunciation.into(), accent_type)
	}
}

pub(crate) struct WavData {
	length: usize,
	data: *mut u8,
}

// safety:
// length/dataはアクセス中に他スレッドからのアクセスを許可してはならない。
// ただし、単一の存在で存在できるのであればその限りではない。
unsafe impl Send for WavData {}

impl WavData {
	fn new(length: usize, data: *mut u8) -> Self {
		Self {
			length,
			data
		}
	}
}

impl AsRef<[u8]> for WavData {
	fn as_ref(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.data, self.length) }
	}
}

impl<'a> IntoIterator for &'a WavData {
	type Item = &'a u8;

	type IntoIter = std::slice::Iter<'a, u8>;

	fn into_iter(self) -> Self::IntoIter {
		self.as_ref().into_iter()
	}
}

impl Drop for WavData {
	fn drop(&mut self) {
		unsafe { voicevox_wav_free(self.data) };
	}
}

pub(crate) struct TtsAudioOptionBuilder {
	option: TtsAudioOption,
}

impl TtsAudioOptionBuilder {
	pub(crate) fn new() -> Self {
		Self {
			option: TtsAudioOption::default()
		}
	}

	pub(crate) fn build(self) -> TtsAudioOption {
		self.option
	}

	pub(crate) fn speed_scale(mut self, value: f64) -> Self {
		self.option.speed_scale = Some(value);
		self
	}

	pub(crate) fn pitch_scale(mut self, value: f64) -> Self {
		self.option.pitch_scale = Some(value);
		self
	}

	pub(crate) fn intonation_scale(mut self, value: f64) -> Self {
		self.option.intonation_scale = Some(value);
		self
	}

	pub(crate) fn volume_scale(mut self, value: f64) -> Self {
		self.option.volume_scale = Some(value);
		self
	}

	pub(crate) fn pause_length_scale(mut self, value: f64) -> Self {
		self.option.pause_length_scale = Some(value);
		self
	}

	pub(crate) fn pre_phoneme_length(mut self, value: f64) -> Self {
		self.option.pre_phoneme_length = Some(value);
		self
	}

	pub(crate) fn post_phoneme_length(mut self, value: f64) -> Self {
		self.option.post_phoneme_length = Some(value);
		self
	}

	pub(crate) fn output_sampling_rate(mut self, value: i32) -> Self {
		self.option.output_sampling_rate = Some(value);
		self
	}

	pub(crate) fn output_stereo(mut self, value: bool) -> Self {
		self.option.output_stereo = Some(value);
		self
	}
}

#[derive(Default)]
pub(crate) struct TtsAudioOption {
	speed_scale: Option<f64>,
	pitch_scale: Option<f64>,
	intonation_scale: Option<f64>,
	volume_scale: Option<f64>,
	pause_length_scale: Option<f64>,
	pre_phoneme_length: Option<f64>,
	post_phoneme_length: Option<f64>,
	output_sampling_rate: Option<i32>,
	output_stereo: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioQuery {
	#[serde(rename = "accent_phrases")]
	accent_phrases: Vec<AccentPhrase>,
	speed_scale: f64,
	pitch_scale: f64,
	intonation_scale: f64,
	volume_scale: f64,
	pre_phoneme_length: f64,
	post_phoneme_length: f64,
	output_sampling_rate: i32,
	output_stereo: bool,
	kana: Option<String>
}

#[derive(Serialize, Deserialize)]
struct AccentPhrase {
	moras: Vec<Mora>,
	accent: i32,
	pause_mora: Option<Mora>,
	is_interrogative: bool,
}

#[derive(Serialize, Deserialize)]
struct Mora {
	text: String,
	vowel: String,
	vowel_length: f64,
	pitch: f64,
	consonant: Option<String>,
	consonant_length: Option<f64>,
}
