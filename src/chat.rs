use entity::enums::Gender;
use openai_dive::v1::{api::Client, error::APIError, resources::{response::{request::{ContentInput, InputMessage, ResponseInput, ResponseInputItem, ResponseParametersBuilder}, response::{OutputContent, ResponseOutput, ResponseText, Role}, shared::{ResponseFormat, ResponseTool, UserLocationType, WebSearchUserLocation}}, shared::WebSearchContextSize}};
use param::ResponseData;
use prompt::SYSTEM_PROMPT;

use crate::STATIC_COMPONENTS;

mod prompt;
pub(crate) mod param;

pub(crate) async fn getchat_responce(user_message: ResponseInputItem, prev_id: Option<String>) -> Result<(ResponseData, String), APIError> {
	let comp_lock = STATIC_COMPONENTS.lock().await;
	let token = comp_lock.get_config().get_chatgpt_token().clone();
	std::mem::drop(comp_lock);

	let client = Client::new(token);
	let responses = client.responses();

	let mut param = ResponseParametersBuilder::default();
	param.model("gpt-4.1".to_string());
	param.instructions(SYSTEM_PROMPT);
	param.input(ResponseInput::List(vec![
		user_message.clone()
	]));
	param.text(ResponseText {
		format: ResponseFormat::Text
	});
	param.tools(vec![
		ResponseTool::WebSearch {
			search_context_size: Some(WebSearchContextSize::Low),
			user_location: Some(WebSearchUserLocation {
				r#type: UserLocationType::Approximate,
				timezone: Some("Asia/Tokyo".to_string()),
				city: None,
				country: None,
				region: None,
			})
		}
	]);
	param.temperature(1.5);
	param.max_output_tokens(10000u32);
	param.top_p(0.9);
	param.store(true);

	if let Some(prev_id) = prev_id {
		param.previous_response_id(prev_id);
	}

	let res = responses.create(param.build().unwrap()).await?;

	let id = res.id;
	let res_output_last = res.output.last().unwrap();
	if let ResponseOutput::Message(message) = res_output_last {
		let content = message.content.first().unwrap();
		if let OutputContent::Text { text, annotations } = content {
			if let Ok(final_res_data) = ResponseData::from_json(&text) {
				Ok((final_res_data, id))
			} else {
				Err(APIError::ParseError("Result message is collapsed".to_string()))
			}
		} else {
			Err(APIError::UnknownError(0, "output content is not text".to_string()))
		}
	} else {
		Err(APIError::UnknownError(0, "response output is not message".to_string()))
	}
}

pub(crate) fn create_user_message<S1: Into<String>, S2: Into<String>>(message: S1, level: u32, name: S2, gender: Gender, time: &chrono::DateTime<chrono::Local>) -> ResponseInputItem {
	let mut res = String::new();

	res += &format!("好感度レベル: {}\n", level);
	res += &format!("名前: {}\n", name.into());
	res += &format!("性別: {}\n", gender.to_string());
	res += &format!("時刻: {}\n", time.format("%H:%M"));

	res += "####\n";
	res += &message.into();

	ResponseInputItem::Message(InputMessage {
		content: ContentInput::Text(res),
		role: Role::User
	})
}
