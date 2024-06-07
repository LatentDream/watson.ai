// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use audio::recorder::InnerRecordingState;
use model::SettingController;
use utils::{filesys::local_data_dir_path, session::InnerSessionState};
use tauri_plugin_log::{LogTarget, fern::colors::ColoredLevelConfig};
use crate::model::MeetingController;
use std::sync::{Mutex, Arc};


mod audio;
mod model;
mod summarizer;
mod ipc;
mod crm;
mod utils;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
fn exit(window: tauri::Window) {
    println!("Exiting");
    let _ = window.close();
}

pub struct RecordingState(pub Arc<Mutex<InnerRecordingState>>);
pub struct SettingControllerState(pub Arc<Mutex<SettingController>>); 
pub struct MeetingControllerState(pub Arc<Mutex<MeetingController>>);
pub struct SeesionState(pub Mutex<InnerSessionState>);


fn main() {

  let log = tauri_plugin_log::Builder::default()
  .targets([
    LogTarget::Folder(local_data_dir_path()),
    LogTarget::Stdout,
    LogTarget::Webview,
  ])
  .with_colors(ColoredLevelConfig::default())
  .level(log::LevelFilter::Debug);

  tauri::Builder::default()
      .plugin(log.build())
      .manage(RecordingState(Arc::new(Mutex::new(InnerRecordingState::new()))))
      .manage(MeetingControllerState(Arc::new(Mutex::new(MeetingController::new()))))
      .manage(SettingControllerState(Arc::new(Mutex::new(SettingController::new(model::SettingPath::Default)))))
      .manage(SeesionState(Mutex::new(InnerSessionState::new())))
      .invoke_handler(tauri::generate_handler![
          // Recorder
          ipc::start_recording, 
          ipc::pause_recording, 
          ipc::resume_recording, 
          ipc::stop_recording,
          ipc::get_recording_state,
          ipc::transcribe_recording,
          ipc::get_available_audio_devices,
          ipc::get_recording_device_names,
          // Meeting
          ipc::get_meeting,
          ipc::list_meetings,
          ipc::update_meeting,
          ipc::summarize_meeting,
          ipc::delete_meeting,
          ipc::archive_meeting,
          ipc::improve_note_meeting,
          ipc::async_summarize_meeting,
          ipc::async_improve_note_meeting,
          ipc::increment_async_ops_meeting,
          ipc::decrement_async_ops_meeting,
          ipc::delete_all_meeting,
          ipc::export_all_meeting,
          // Setting
          ipc::get_setting,
          ipc::update_setting,
          ipc::open_data_folder,
          // CRM
          ipc::search_organizations_crm,
          ipc::get_organization_crm,
          ipc::search_persons_crm,
          ipc::publish_summary_crm,
          ipc::list_lists_crm,
          // Session
          ipc::get_new_meeting_note,
          ipc::set_new_meeting_note,
          // Tauri specific
          exit,
          ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

