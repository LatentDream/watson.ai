
use log::info;
/*
    * Information valid for the entire session
    * Won't be cleared until the app is closed
    * Won't be saved to disk
*/
use serde::{Deserialize, Serialize};
use anyhow::Error;
use ts_rs::TS;


#[derive(Clone, Deserialize, Serialize, TS, Debug)]
#[ts(export, export_to = "../src/bindings/")]
pub struct NewMeetingNote {
    pub note: String,
    pub title: String,
}

pub struct InnerSessionState {
    pub new_meeting_note: NewMeetingNote,
}

impl InnerSessionState {
    pub fn new() -> Self {
        Self {
            new_meeting_note: NewMeetingNote {
                note: String::new(),
                title: String::new(),
            },
        }
    }

    pub fn update_new_meeting_note(&mut self, new_meeting_note: NewMeetingNote) -> Result<(), Error> {
        info!("{}", format!("Updated new_meeting_note {:?} -", new_meeting_note));
        
        self.new_meeting_note = new_meeting_note;
        info!("{}", format!("Updated new_meeting_note {} -", self.new_meeting_note.note));
        return Ok(());
    }

    pub fn get_new_meeting_note(&self) -> Result<NewMeetingNote, Error> {
        return Ok(self.new_meeting_note.clone());
    }
}