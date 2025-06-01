use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub(crate) enum Gender {
	Male,
	Ladies,
}

impl Display for Gender {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Gender::Male => write!(f, "M"),
			Gender::Ladies => write!(f, "L"),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResponseData {
	pub(crate) message: String,
	pub(crate) actions: Vec<ResponseAction>,
}

impl ResponseData {
	pub(crate) fn from_json(json: &str) -> Result<Self, serde_json::Error> {
		serde_json::from_str(json)
	}
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResponseAction {
	pub(crate) name: String,
	pub(crate) params: Map<String, Value>,
}
