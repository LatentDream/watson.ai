
use log::{error, info};
use reqwest::{self, StatusCode};
use std::thread::sleep;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::model::{SettingController, Chapter};
use std::io::prelude::*; // for read_to_end()
use anyhow::Error;

#[derive(Deserialize)]
pub struct TranscriptResponse {
    pub id: String,
    pub language_model: Option<String>,
    pub acoustic_model: Option<String>,
    pub language_code: Option<String>,
    pub audio_url: String,
    pub status: String,
    pub text: Option<String>,
    pub chapters: Option<Vec<Chapter>>,
}

#[derive(Deserialize, Serialize)]
struct StartingTranscriptResponse {
    id: String,
}

#[derive(Deserialize, Serialize)]
struct UploadResponse {
    upload_url: String,
}

pub fn get_transcript(audio_path: &String, language: Option<String>) -> Result<String, Error> {
    let result = transcribe_with_assemblyai(audio_path, language);
    match result {
        Ok(transcript_response) => {
            let transcript = transcript_response.text.unwrap();
            return Ok(transcript);
        }
        Err(error) => {
            error!("Async Transcription failed: {:?}", error);
            return Err(error);
        }
    }
}

fn transcribe_with_assemblyai(audio_path: &String, language: Option<String>) -> Result<TranscriptResponse, Error> {
    let base_url = "https://api.assemblyai.com/v2";

    let setting = SettingController::new(crate::model::SettingPath::Default).get_setting().unwrap();
    let assemblyai_api_token = setting.assemblyai_api_token;

    // Read the audio file
    let mut file = std::fs::File::open(audio_path);
    let mut file = match file {
        Ok(file) => file,
        Err(error) => {
            error!("Error opening audio file: {:?}", error);
            return Err(anyhow::Error::msg("Transcription failed, cannot find the associated audio"));
        }
    };
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;

    info!("Uploading audio to assemblyAI...");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!("{}/upload", base_url))
        .header("authorization", assemblyai_api_token.clone())
        .body(contents)
        .send()?;

    if response.status() != StatusCode::OK {
        error!("Uploading audio to AssemblyAI failed with status code: {:?} - {:?}", response.status(),  response.text());
        return Err(anyhow::Error::msg("Transcription failed while uploading audio"));
    }

    let upload_url = response.json::<UploadResponse>()?.upload_url
        .as_str()
        .to_owned();

    // Determine language for transcription
    let language: String = match language {
        Some(language) => {
            match language.to_lowercase().as_str() {
                "en_us" | "en" | "english" | "anglais" => "en_us".to_string(),
                "fr" | "french" | "francais" | "français" => "fr".to_string(),
                "zh" | "chinese" | "中文" => "zh".to_string(),
                _ => "en_us".to_string()
            }
        }
        None => "en_us".to_string()
    };
        
    // Prepare transcription data
    let data = serde_json::json!({
        "audio_url": upload_url,
        "auto_chapters": if language == String::from("en_us") { true } else { false },
        "language_code": language,
    });

    info!("Start transcription...");
    // Start transcription
    let response = client
        .post(&format!("{}/transcript", base_url))
        .header("authorization", assemblyai_api_token.clone())
        .json(&data)
        .send()?;

    if response.status() != StatusCode::OK {
        error!("Starting request for transcription with assemblyAI failed with status code: {:?} - {:?}", response.status(),  response.text());
        return Err(anyhow::Error::msg("Transcription failed while starting transcription"));
    }

    let transcript_id = response.json::<StartingTranscriptResponse>()?.id
        .as_str()
        .to_owned();

    let polling_endpoint = format!("{}/transcript/{}", base_url, transcript_id);
    info!("Start polling...");

    loop {
        let response = client
            .get(&polling_endpoint)
            .header("authorization", assemblyai_api_token.clone())
            .send()?;

        if response.status() != StatusCode::OK {
            error!("Request failed with status code: {:?} - {:?}", response.status(),  response.text());
            return Err(anyhow::Error::msg("Transcription failed while polling transcription"));
        }
        
        let transcription_result: TranscriptResponse = response.json()?;

        match transcription_result.status.as_str() {
            "completed" => {
                info!("Transcription completed!");
                return Ok(transcription_result);
            }
            "error" => {
                let transcription_id = transcription_result.id;
                error!("Transcription with assemblyAI failed - id: {:?}", transcription_id);
                return Err(anyhow::Error::msg("Transcription failed"));
            }
            _ => {
                sleep(Duration::from_secs(3));
                info!("Waiting for AssemblyAI to complete transcription...");
            }
        }
    }
}
