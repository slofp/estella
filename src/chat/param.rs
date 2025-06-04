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
