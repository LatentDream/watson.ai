
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::{fs::File, io::Read, io::Write, io};
use crate::ipc::ModelMutateResultData;
use crate::model::Meeting;
use crate::utils::filesys::local_data_dir_path;
use anyhow::Error;
use log::{ warn, error, info};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::path::{Path, PathBuf};
use zip::{ZipWriter, CompressionMethod, write::FileOptions};

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct MeetingsRef {
    pub uuid: String,
    pub title: String,
    pub datetime: String,
    pub number_ops: i32,
}

fn default_number_ops() -> i32 {0}

#[derive(Deserialize, Serialize)]
pub struct MeetingInfo {
    title: String,
    datetime: String,
    #[serde(default = "default_number_ops")] // Backward compatibility
    number_ops: i32
}

impl MeetingInfo {

    pub fn new(title: String, datetime: String) -> MeetingInfo {
        MeetingInfo {
            title: title,
            datetime: datetime,
            number_ops: 0
        }
    }

    pub fn incremeent_async_ops(&mut self) {
        self.number_ops = self.number_ops + 1;
    }
    
    pub fn decremeent_async_ops(&mut self) {
        if self.number_ops > 0 {
            self.number_ops = self.number_ops - 1;
        } else {
            self.number_ops = 0;
        }
    }
}

pub struct MeetingController {
    // uuid -> (title, datetime)
    pub meetings: HashMap<String, MeetingInfo>,
}

impl MeetingController {
    pub fn new() -> MeetingController {
        println!("MeetingController::new");
        let local_data_path = local_data_dir_path().join("meetingsRef.json").to_str().unwrap().to_string();        
        let file = File::open(local_data_path);
        let mut s = Self {
            meetings: HashMap::new()
        };
        println!("Loading meetingsRef file");
        match file { 
            Ok(mut file) => {
                let mut contents: String = String::new();
                let _ = file.read_to_string(&mut contents);
                match serde_json::from_str(&contents) {
                    Ok(meetings) => {
                        println!("Loaded meetingsRef file\n");
                        s.meetings = meetings;
                        // All number of async ops should be 0 at startup
                        for (_, meeting_info) in s.meetings.iter_mut() {
                            meeting_info.number_ops = 0;
                        }
                    },
                    Err(error) => {
                        println!("Error while parsing file: {:?}\n", error);
                        error!("Error while parsing file: {:?}\n", error);
                    }
                } 
            },
            Err(error) => {
                println!("No local data - Error while loading meetingRef file: {:?}\n", error);
                warn!("No local data - Error while loading meetingRef file: {:?}\n", error);
            }
        }
        return s;
    }

    pub fn save(&self) -> Result<(), Error> {
        let serialized = serde_json::to_string(&self.meetings)?;
        let local_data_path = local_data_dir_path().join("meetingsRef.json").to_str().unwrap().to_string();        
        let parent_dir = Path::new(&local_data_path).parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }
        let mut file = File::create(&local_data_path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<MeetingsRef>, Error> {
        let mut list = Vec::new();
        for (uuid, meeting_info) in self.meetings.iter() {
            let meeting_ref = MeetingsRef {
                uuid: uuid.to_string(),
                title: meeting_info.title.to_string(),
                datetime: meeting_info.datetime.to_string(),
                number_ops: meeting_info.number_ops.clone()

            };
            list.push(meeting_ref);
        }
        return Ok(list);
    }

    pub fn get(&self, uuid: String) -> Result<Meeting, Error> {
        info!("Loading meeting - id {}", uuid);
        return Meeting::load(uuid);
    }

    pub fn update(&mut self, meeting: Meeting) -> Result<ModelMutateResultData, Error> {
        if !self.meetings.contains_key(&meeting.get_uuid()) {
            return Err(anyhow::anyhow!("Meeting does not exist"));
        }
        let _ = self.meetings.insert(
            meeting.get_uuid(), 
            MeetingInfo {
                title: meeting.title.clone(),
                datetime: meeting.datetime.clone(),
                number_ops: self.meetings.get(&meeting.get_uuid()).unwrap().number_ops.clone()
            });
        let _ = meeting.save();
        self.save()?;
        return Ok(ModelMutateResultData { id: meeting.get_uuid() });
    }

    pub fn add(&mut self, meeting: Meeting) -> Result<Meeting, Error> {
        if self.meetings.contains_key(&meeting.get_uuid()) {
            return Err(anyhow::anyhow!("Meeting already exists"));
        }
        self.meetings.insert(meeting.get_uuid(), MeetingInfo::new(meeting.title.clone(), meeting.datetime.clone()));
        meeting.save()?;
        self.save()?;
        return Ok(meeting);
    }

    pub fn delete(&mut self, uuid: String) -> Result<ModelMutateResultData, Error> {
        // Meeting will be deleted from disk
        if !self.meetings.contains_key(&uuid) {
            return Err(anyhow::anyhow!("Meeting does not exist"));
        }
        self.meetings.remove(&uuid);
        let mut meeting = Meeting::load(uuid.clone())?;
        meeting.delete_from_disk();
        self.save()?;
        return Ok(ModelMutateResultData { id: uuid });
    }

    pub fn delete_all(&mut self) -> Result<(), Error> {
        // Meeting will be deleted from disk
        for (uuid, _) in self.meetings.iter() {
            let mut meeting = Meeting::load(uuid.clone())?;
            meeting.delete_from_disk();
        }
        self.meetings.clear();
        self.save()?;
        return Ok(());
    }

    pub fn archive(&mut self, uuid: String) -> Result<ModelMutateResultData, Error> {
        // Meeting will never be shown again, but still exist on disk
        if !self.meetings.contains_key(&uuid) {
            return Err(anyhow::anyhow!("Meeting does not exist"));
        }
        self.meetings.remove(&uuid);
        self.save()?;
        return Ok(ModelMutateResultData { id: uuid });
    }

    pub fn archive_all(&mut self) -> Result<(), Error> {
        // Meeting will never be shown again, but still exist on disk
        self.meetings.clear();
        self.save()?;
        return Ok(());
    }

    pub fn export_all(&self) -> Result<String, Error> {
        // Create a zip file with all meetings and audio files on disk
        // -> Return the path to the zip file created
        let now = chrono::Utc::now();
        let date = now.format("%Y-%m-%d_%H-%M-%S");
        let binding = local_data_dir_path().join("export").join(format!("compressed_meetings_{}.zip", date));
        // Add current date to pathbuf        
        let zip_file_path = binding.as_path();
        let parent_dir = zip_file_path.parent().unwrap();
        if !parent_dir.exists() {std::fs::create_dir_all(parent_dir)?;}
        let zip_file = File::create(&zip_file_path)?;
        let mut files_to_compress: Vec<PathBuf> = vec![];
        let mut zip = ZipWriter::new(zip_file);
        let options = FileOptions::default().compression_method(CompressionMethod::DEFLATE); 

        // Files to compress.
        let folder_path = local_data_dir_path().join("meetings");
        let dir_entries = fs::read_dir(folder_path)?;
        for entry in dir_entries {
            let entry = entry?;
            files_to_compress.push(entry.path());
        }
        let folder_path = local_data_dir_path().join("audio");
        let dir_entries = fs::read_dir(folder_path)?;
        for entry in dir_entries {
            let entry = entry?;
            files_to_compress.push(entry.path());
        }
    
        // Iterate through the files and add them to the ZIP archive.
        for file_path in &files_to_compress {
            let file = File::open(file_path)?;
            let file_name = file_path.file_name().unwrap().to_str().unwrap();
            zip.start_file(file_name, options)?;
            let mut buffer = Vec::new();
            io::copy(&mut file.take(u64::MAX), &mut buffer)?;
            zip.write_all(&buffer)?;
        }
        zip.finish()?;
        info!("Files compressed successfully to {:?}", zip_file_path); 

        // Open the folder containing the zip file for the user to find it
        #[cfg(target_family = "windows")]
        Command::new("explorer")
        .args(["/select,", zip_file_path.to_str().unwrap()]) // The comma after select is not a typo
        .spawn()
        .unwrap();

        #[cfg(target_os = "macos")]
        Command::new("open")
        .args(["-R", zip_file_path.to_str().unwrap()])
        .spawn()
        .unwrap();

        return Ok(zip_file_path.to_str().unwrap().to_string());
    }


    pub fn summarize_meeting(&mut self, uuid: String) -> Result<ModelMutateResultData, Error> {
        let mut meeting = self.get(uuid.clone())?;
        meeting.summarize()?;
        self.update(meeting)
    }

    pub fn improve_note_meeting(&mut self, uuid: String) -> Result<ModelMutateResultData, Error> {
        let mut meeting = self.get(uuid.clone())?;
        meeting.improve_note()?;
        self.update(meeting)
    }

    // Async ops - Used to know if a meeting is being processed by a background task (Used in UI to display a spinner)

    pub fn increment_async_ops(&mut self, uuid: String) -> Result<ModelMutateResultData, Error> {
        if !self.meetings.contains_key(&uuid) {
            return Err(anyhow::anyhow!("Meeting does not exist"));
        }
        let meeting_info = self.meetings.get_mut(&uuid).unwrap();
        meeting_info.incremeent_async_ops();
        return Ok(ModelMutateResultData { id: uuid });
    }

    pub fn decrement_async_ops(&mut self, uuid: String) -> Result<ModelMutateResultData, Error> {
        if !self.meetings.contains_key(&uuid) {
            return Err(anyhow::anyhow!("Meeting does not exist"));
        }
        let meeting_info = self.meetings.get_mut(&uuid).unwrap();
        meeting_info.decremeent_async_ops();
        return Ok(ModelMutateResultData { id: uuid });
    }

    pub fn get_nb_async_ops(&self, uuid: String) -> Result<i32, Error> {
        if !self.meetings.contains_key(&uuid) {
            return Err(anyhow::anyhow!("Meeting does not exist"));
        }
        let meeting_info = self.meetings.get(&uuid).unwrap();
        return Ok(meeting_info.number_ops.clone());
    }

}
