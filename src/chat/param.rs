use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ResponseData {
	pub(crate) message: String,
	pub(crate) actions: Vec<ResponseAction>,
}

impl ResponseData {
	pub(crate) fn from_json(json: &str) -> Result<Self, serde_json::Error> {
		serde_json::from_str(json)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ResponseAction {
	pub(crate) name: String,
	pub(crate) params: Map<String, Value>,
}

pub(crate) fn calculate_likability_level_from_message_count(value: u32) -> u32 {
	if value <= 10 {
		value
	} else if value <= 175 {
		((9.0f32 + 16.0f32 * value.to_f32().unwrap_or_default()).sqrt() - 3.0f32).floor().to_u32().unwrap_or_default()
	} else {
		((((value.to_f32().unwrap_or_default() / 175.0f32).sqrt() / 1.0f32.sqrt()) - 1.0f32) * 35.967f32 + 50.0f32).floor().to_u32().unwrap_or_default().min(100)
	}
}
