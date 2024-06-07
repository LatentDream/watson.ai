use std::{fs::File, io::Write, path::Path, process::Command};
use crate::summarizer::openai;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use anyhow::Error;
use uuid::Uuid;
use log::{info, warn};
use crate::utils::filesys::local_data_dir_path;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Prompt {
    pub name: String,
    pub prompt: String,
}


#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct Setting {
    uuid: String, // To have a "standard" id field for all models.
    pub assemblyai_api_token: String,
    pub openai_api_token: String,
    pub affinity_api_token: String,
    pub affinity_crm_list_id: Option<String>,
    pub prompts: Option<Vec<Prompt>>,
    pub default_model: Option<openai::ModelTurbo>,
}

impl Setting {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            assemblyai_api_token: String::new(),
            openai_api_token: String::new(),
            affinity_api_token: String::new(),
            affinity_crm_list_id: None,
            prompts: Some(vec![
                Prompt { name: String::from("VC Intro Call"), prompt: String::from("Extract key details from your introductory call, ensuring accuracy and conciseness. Ignore the information about our fund, focus on the startup:\n- General discussion points\n- Team overview\n- Company's core activities\n- Problem addressed by the company\n- Target market and sales approach\n- Competitive advantages\n- Business model overview\n- Current status of the company\n- Funding status or recent fundraising efforts\n- Action items, if assigned")},
                Prompt { name: String::from("Q&A Call"), prompt: String::from("Extract and organize questions and answers from the call in a structured format. Be accurate and use 'N/A' if information is not applicable or unknown.")}
                ]),
            default_model: Some(openai::ModelTurbo::GPT4),
        }
    }
}

pub struct SettingController {
    settings: Setting,
    local_data_path: String,
}

pub enum SettingPath {
    Default,
    Custom(String),
}

impl SettingController {
    pub fn new(local_data_path: SettingPath) -> Self {
        let path = match local_data_path {
            SettingPath::Default => local_data_dir_path().join("settings.json").to_str().unwrap().to_string(),
            SettingPath::Custom(path) => path,
        };
        let mut s = Self {
            settings: Setting::new(),
            local_data_path: path
        };
        s.load_data();
        return s;
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string(&self.settings)?;
        let parent_dir = Path::new(&self.local_data_path).parent().unwrap();
        // Create the parent directories if they don't exist
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }
        let mut file = File::create(&self.local_data_path)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }

    fn load_data(&mut self) {
        let file = File::open(&self.local_data_path);
        info!("Loading settings from {:?}", &self.local_data_path);        
        match file {
            Ok(file) => {
                let prompts = self.settings.prompts.clone();
                let mut settings: Setting = serde_json::from_reader(file).unwrap();
                if settings.prompts.is_none() {
                    // Alway provide the default prompts if the file doesn't contain any.
                    settings.prompts = prompts;
                }
                self.settings = settings;
            },
            Err(error) => {
                warn!("No local data detected - Error while loading file: {:?}\n", error);
            }
        }
    }

    pub fn get_setting(&self) -> Result<Setting, Error> {
        return Ok(self.settings.clone());
    }

    pub fn set_setting(&mut self, setting: Setting) -> Result<(), Error> {
        self.settings = setting;
        let _ = self.save();
        return Ok(());
    }

    pub fn open_data_folder(&self) -> Result<(), Error> {
        let path = Path::new(&self.local_data_path);
        // Open the folder containing the zip file for the user to find it
        #[cfg(target_family = "windows")]
        Command::new("explorer")
        .args(["/select,", path.to_str().unwrap()]) // The comma after select is not a typo
        .spawn()
        .unwrap();

        #[cfg(target_os = "macos")]
        Command::new("open")
        .args(["-R", path.to_str().unwrap()])
        .spawn()
        .unwrap();
    
        return Ok(());
    }

}