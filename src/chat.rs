use chrono::Utc;
use entity::{enums::Gender, talk_history};
use openai_dive::v1::{api::Client, error::APIError, resources::{response::{request::{ContentInput, InputMessage, ResponseInput, ResponseInputItem, ResponseParametersBuilder}, response::{OutputContent, ResponseOutput, ResponseText, Role}, shared::{ResponseFormat, ResponseTool, UserLocationType, WebSearchUserLocation}}, shared::WebSearchContextSize}};
use param::ResponseData;
use prompt::SYSTEM_PROMPT;
use sea_orm::Set;
use sea_orm::ActiveModelTrait;

use crate::STATIC_COMPONENTS;

mod prompt;
pub(crate) mod param;

pub(crate) async fn getchat_responce(user_id: u64, user_message: ResponseInputItem, prev_id: Option<String>) -> Result<(ResponseData, String), APIError> {
	let input_message = if let ResponseInputItem::Message(v) = &user_message {
		if let ContentInput::Text(t) = &v.content {
			t.clone()
		} else {
			String::new()
		}
	} else {
		String::new()
	};

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
	param.temperature(1.0);
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
			// DBに会話内容を保存
			let th = talk_history::ActiveModel {
				chat_id: Set(id.clone()),
				input_text: Set(input_message),
				output_text: Set(text.clone()),
				user_id: Set(user_id),
				// UTC?
				talk_date: Set(Utc::now().naive_utc()),
				..Default::default()
			};
			let lsc = STATIC_COMPONENTS.lock().await;
			let mysql_client = lsc.get_sql_client();
			let _ = th.insert(mysql_client).await.inspect_err(|e| {
				log::error!("{:?}", e);
			});
			std::mem::drop(lsc);

			// パースできたら返す
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

pub(crate) fn create_user_message<S1: Into<String>, S2: Into<String>>(message: S1, level: u32, userId: u64, name: S2, gender: Gender, time: &chrono::DateTime<chrono::Local>) -> ResponseInputItem {
	let mut res = String::new();

	res += &format!("好感度レベル: {}\n", level);
	res += &format!("ID: {}\n", userId);
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

pub(crate) fn create_multi_user_message(items: Vec<&ResponseInputItem>) -> ResponseInputItem {
	let mut text = String::new();
	let mut first = true;
	for item in items {
		if let ResponseInputItem::Message(v) = item {
			if let ContentInput::Text(v) = &v.content {
				if first {
					first = false;
				} else {
					text += "\n";
					text += "----\n";
				}

				text += v;
			}
		}
	}

	ResponseInputItem::Message(InputMessage {
		content: ContentInput::Text(text),
		role: Role::User
	})
}
