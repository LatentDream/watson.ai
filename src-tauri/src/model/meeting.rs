/*
    * Meeeting Model
    * This model is used to store the meeting data
*/
use std::{fs::File, io::Read, io::Write};
use anyhow::Error;
use serde_with_macros::skip_serializing_none;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::utils::filesys::local_data_dir_path; 
use crate::summarizer::openai::summarize_with_openai;
use std::path::Path;
use log::{info, warn, error};


#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Meeting {
    uuid: String,
    pub title: String,
    pub company_name: String,
    pub company_id: String,
    pub prompt: String,
    pub summary: String,
    pub note: String,
    pub transcript: String,
    pub datetime: String,
    pub audio_path: String,
    pub published: bool,
    pub publish_with_note: Option<bool>,
    chapters: Vec<Chapter>,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Chapter {
    summary: String,
    gist: String,
    headline: String,
    start: i32,
    end: i32,
}


impl Meeting {
    pub fn new(uuid: Option<Uuid>, title:String, datetime: chrono::DateTime<Utc>, audio_path: String) -> Self {
        let s = Self {
            uuid: uuid.unwrap_or(Uuid::new_v4()).to_string(),
            title,
            company_name: String::new(), 
            company_id: String::new(),
            prompt: String::new(),
            summary: String::new(),
            note: String::new(),
            transcript: String::new(),
            datetime: datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            audio_path,
            published: false,
            publish_with_note: Some(false),
            chapters: Vec::new(),
        };
        let _ = s.save();
        return s;
    }

    pub fn load(uuid: String) -> Result<Self, Error> {
        let meeting_path = local_data_dir_path().join("meetings").join(format!("{}.json", uuid)).to_str().unwrap().to_string();
        let file = File::open(meeting_path);
        match file {
            Ok(mut file) => {
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);
                match serde_json::from_str(&contents) {
                    Ok(meeting) => {
                        return Ok(meeting);
                    },
                    Err(error) => {
                        error!("Error while parsing file {}.json: {:?}\n", uuid, error);
                        return Err(error.into());
                    }
                } 
            },
            Err(error) => {
                warn!("No local data detected - Error while loading file {}.json: {:?}\n", uuid, error);
                return Err(error.into());
            }
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        let meeting_path = local_data_dir_path().join("meetings").join(format!("{}.json", self.uuid)).to_str().unwrap().to_string();
        let parent_dir = Path::new(&meeting_path).parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }
        let file = File::create(meeting_path);
        match file {
            Ok(mut file) => {
                let serialized = serde_json::to_string(&self)?;
                file.write_all(serialized.as_bytes())?;
                return Ok(());
            },
            Err(error) => {
                error!("Error while saving file {}.json: {:?}\n", self.uuid, error);
                return Err(error.into());
            }            
        }
    }

    pub fn delete_from_disk(&mut self) {
        let audio_path = local_data_dir_path().join("audio").join(self.audio_path.clone()).to_str().unwrap().to_string();
        let result = std::fs::remove_file(audio_path);
        match result {
            Ok(_) => {
                info!("Audio file {} deleted successfully", self.audio_path);
            },
            Err(error) => {
                warn!("Error while deleting audio file {}: {:?}", self.audio_path, error);
            }
        }
        let meeting_path = local_data_dir_path().join("meetings").join(format!("{}.json", self.uuid)).to_str().unwrap().to_string();
        let result = std::fs::remove_file(meeting_path);
        match result {
            Ok(_) => {
                info!("Meeting file {}.json deleted successfully", self.uuid);
            },
            Err(error) => {
                warn!("Error while deleting meeting file {}.json: {:?}", self.uuid, error);
            }
        }
    }

    pub fn get_uuid(&self) -> String {
        return self.uuid.clone();
    }

    pub fn summarize(&mut self) -> Result<(), Error>{
        if !self.transcript.is_empty() {
            let prompt = match self.prompt.is_empty() {
                true => None,
                false => Some(self.prompt.clone())
            };
            match summarize_with_openai(self.transcript.clone(), prompt) {
                Ok(summary) => {
                    self.summary = summary;
                },
                Err(error) => {
                    error!("Error while summarizing meeting {}: {:?}", self.uuid, error);
                    return Err(anyhow::anyhow!(error));
                }
            }
            info!("{}", format!("Summary generated successfully: {}", self.summary));
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("No transcript to summarize"));
        }
    }

    pub fn improve_note(&mut self) -> Result<(), Error> {
        if !self.note.is_empty() {
            let prompt = format!("Refine and complete the note with missing information, maintaining a similar structure in HTML format. This is crucial for accurate documentation. <note>{}</note>", self.note);
            match summarize_with_openai(self.transcript.clone(), Some(prompt)) {
                Ok(note) => {
                    self.summary = note;
                    return Ok(());
                },
                Err(error) => {
                    error!("Error while improving note of meeting {}: {:?}", self.uuid, error);
                    return Err(anyhow::anyhow!(error));
                }
            }
        } else {
            return Err(anyhow::anyhow!("No note to improve"));
        }
    }

}

#[skip_serializing_none]
#[derive(Deserialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct MeetingForUpdate {
	pub meeting: Meeting,
}
