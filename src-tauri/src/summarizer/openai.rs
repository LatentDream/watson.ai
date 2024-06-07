use anyhow::Error;
use openai_api_rust::*;
use openai_api_rust::chat::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::model::SettingController;
use log::{error, info};

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub enum ModelTurbo {
    GPT3,
    GPT4,
}

impl ModelTurbo {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelTurbo::GPT3 => "gpt-3.5-turbo-1106",
            ModelTurbo::GPT4 => "gpt-4-1106-preview",
        }
    }
}


pub fn summarize_with_openai(transcript: String, prompt: Option<String>) -> Result<String, Error> {

    let setting = SettingController::new(crate::model::SettingPath::Default).get_setting().unwrap();
    let model = match setting.default_model {
        Some(model) => model,
        None => ModelTurbo::GPT3,
    };
    info!("OpenAI model: {}", model.as_str());
    let format = String::from("[VERY IMPORTANT: Answer in HTML format directly, Without the header <!DOCTYPE html><html><head> ...</head>), don't include <body> tag don't use <h1> tag, prefer using <h4> and <li> tags instead");

    let prompt = match prompt {
        Some(prompt) => format!("<Instruction>{} \n&\n {} \n</Instruction> <transcript>{}/transcipt>", prompt, format, transcript),
        None => format!("Refine and organize the provided transcript using bullet points. \n&\n {} \n<transcript>{}</transcript>", format, transcript)
    };

    let auth = Auth::new(&setting.openai_api_token);
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body: ChatBody = ChatBody {
        model: model.as_str().to_string(),
        max_tokens: None,
        temperature: Some(0_f32),
        top_p: Some(0_f32),
        n: Some(2),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message { role: Role::User, content: prompt }],
    };
    let rs = openai.chat_completion_create(&body);
    info!("OpenAI response: {:?}", rs);
    let choice = match rs {
        Ok(rs) => rs.choices,
        Err(e) => {
            error!("Error in transcription: {:?}", e);
            return Err(anyhow::anyhow!("Error occured in the summarization: {:?}", e));
        }
    };
    let message = &choice[0].message.as_ref().unwrap();

    return Ok(message.content.clone());
}
