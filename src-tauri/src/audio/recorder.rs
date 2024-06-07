#[cfg(target_os = "macos")]
use super::screen_capture_kit;
#[cfg(target_os = "macos")]
use objc_id::Id;
#[cfg(target_os = "macos")]
use screencapturekit_sys::stream::UnsafeSCStream;
use super::cpal_audio::{cpal_utils, microphone, speakers};
use crate::model::Meeting;
use crate::utils::filesys::local_data_dir_path;
use anyhow::Error;
use chrono::Utc;
use log::{error, info};
use serde::Serialize;
use std::{
    path::Path,
    sync::mpsc::{channel, Sender},
};
use tauri::Window;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, TS)]
pub enum State {
    Stopped,
    Recording,
    Paused,
}

pub struct InnerRecordingState {
    state: State,
    outout_sender: Sender<cpal_utils::RecordingMessage>,
    input_sender: Sender<cpal_utils::RecordingMessage>,
    output_device_name: String,
    input_device_name: String,
    speakers_output_path: String,
    microphone_output_path: String,
    starting_time: chrono::DateTime<Utc>,
    uuid: Uuid,
    // TODO: Change to cpal with integration cpal <> screen capture kit is done
    #[cfg(target_os = "macos")]
    sc_stream: Option<Id<UnsafeSCStream>>,
}

impl InnerRecordingState {
    pub fn new() -> Self {
        // Communication channels between the recording thread and the main thread
        let (outout_sender, _) = channel();
        let (input_sender, _) = channel();

        // Temporary files to store the recording
        let speakers_output_path = local_data_dir_path()
            .join("tmp")
            .join("speaker_recorded.wav")
            .to_str()
            .unwrap()
            .to_string();
        let microphone_output_path = local_data_dir_path()
            .join("tmp")
            .join("microphone_recorded.wav")
            .to_str()
            .unwrap()
            .to_string();
        let parent_dir = Path::new(&microphone_output_path).parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }

        let starting_time = Utc::now();
        let id = Uuid::new_v4();

        // TODO: Change to cpal with integration cpal <> screen capture kit is done
        #[cfg(target_os = "macos")]
        let sc_stream = Some(screen_capture_kit::speaker::init());
        Self {
            state: State::Stopped,
            outout_sender,
            input_sender,
            output_device_name: "".to_string(),
            input_device_name: "".to_string(),
            speakers_output_path,
            microphone_output_path,
            starting_time,
            uuid: id,
            #[cfg(target_os = "macos")]
            sc_stream,
        }
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn get_device_names(&self) -> Result<cpal_utils::RecordingDevices, Error> {
        let output_device_name = self.output_device_name.clone();
        let input_device_name = self.input_device_name.clone();
        return Ok(cpal_utils::RecordingDevices {
            output_device_name,
            input_device_name,
        });
    }

    pub fn start(
        &mut self,
        window: Window,
        input_device_name: Option<String>,
        output_device_name: Option<String>,
    ) -> Result<String, Error> {
        match self.state {
            State::Stopped => {
                self.state = State::Recording;
                self.starting_time = Utc::now();
                self.uuid = Uuid::new_v4();
                // TODO: Change to cpal with integration cpal <> screen capture kit is done
                #[cfg(target_os = "macos")]
                {
                    if self.sc_stream.is_none() {
                        self.sc_stream = Some(screen_capture_kit::speaker::init());
                    }
                    screen_capture_kit::speaker::start_capture(self.sc_stream.as_ref().unwrap());
                }
                #[cfg(not(target_os = "macos"))]
                {
                    match speakers::record(
                        &self.speakers_output_path,
                        window.clone(),
                        output_device_name,
                    ) {
                        Ok(communication_channel) => {
                            self.outout_sender = communication_channel.sender;
                            self.output_device_name = communication_channel.target_device;
                        }
                        Err(error) => {
                            error!("Error while starting recording of speaker: {:?}", error);
                            return Err(anyhow::anyhow!(error));
                        }
                    }
                }
                match microphone::record(&self.microphone_output_path, window, input_device_name) {
                    Ok(communication_channel) => {
                        self.input_sender = communication_channel.sender;
                        self.input_device_name = communication_channel.target_device;
                    }
                    Err(error) => {
                        error!("Error while starting recording of microphone: {:?}", error);
                        return Err(anyhow::anyhow!(error));
                    }
                }
                return Ok(format!(
                    "Listening on {} and {}",
                    self.output_device_name, self.input_device_name
                ));
            }
            _ => {
                return Err(anyhow::anyhow!("Recording already started"));
            }
        }
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        match self.state {
            State::Recording => {
                self.state = State::Paused;
                // TODO: Change to cpal with integration cpal <> screen capture kit is done
                #[cfg(target_os = "macos")]
                {
                    screen_capture_kit::speaker::pause_capture(self.sc_stream.as_ref().unwrap());
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.outout_sender
                        .send(cpal_utils::RecordingMessage::Pause)
                        .expect("Failed to send Pause message - Speakers");
                }
                self.input_sender
                    .send(cpal_utils::RecordingMessage::Pause)
                    .expect("Failed to send Pause message - Microphone");
                info!("Paused");
                return Ok(());
            }
            State::Stopped => return Err(anyhow::anyhow!("Recording not started")),
            State::Paused => return Err(anyhow::anyhow!("Recording already paused")),
        }
    }

    pub fn resume(&mut self) -> Result<(), Error> {
        match self.state {
            State::Paused => {
                self.state = State::Recording;
                // TODO: Change to cpal with integration cpal <> screen capture kit is done
                #[cfg(target_os = "macos")]
                {
                    screen_capture_kit::speaker::resume_capture(self.sc_stream.as_ref().unwrap());
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.outout_sender
                        .send(cpal_utils::RecordingMessage::Resume)
                        .expect("Failed to send Resume message - Speakers");
                }
                self.input_sender
                    .send(cpal_utils::RecordingMessage::Resume)
                    .expect("Failed to send Resume message - Microphone");
                info!("resumed");
                return Ok(());
            }
            State::Stopped => return Err(anyhow::anyhow!("Recording not started")),
            State::Recording => return Err(anyhow::anyhow!("Recording not paused")),
        }
    }

    pub fn stop(&mut self) -> Result<Meeting, Error> {
        match self.state {
            State::Recording | State::Paused => {
                self.state = State::Stopped;
                info!("[Recorder] Stopping - {}", self.uuid);
                // TODO: Change to cpal with integration cpal <> screen capture kit is done
                #[cfg(target_os = "macos")]
                {
                    screen_capture_kit::speaker::stop_capture(self.sc_stream.as_ref().unwrap());
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.outout_sender
                        .send(cpal_utils::RecordingMessage::Stop)
                        .expect("Failed to send Stop message - Speakers");
                }
                self.input_sender
                    .send(cpal_utils::RecordingMessage::Stop)
                    .expect("Failed to send Stop message - Microphone");

                // TODO: Change to cpal with integration cpal <> screen capture kit is done
                #[cfg(target_os = "macos")]
                {
                    // We have to convert the raw audio from screen capture kit to wav format for futher processing
                    screen_capture_kit::speaker::convert_to_wav(&self.speakers_output_path);
                }

                let output_path = local_data_dir_path()
                    .join("audio")
                    .join(format!("{}-recording.mp3", self.uuid))
                    .to_str()
                    .unwrap()
                    .to_string();
                let parent_dir = Path::new(&output_path).parent().unwrap();
                if !parent_dir.exists() {
                    std::fs::create_dir_all(parent_dir).unwrap();
                }

                let ffmpegcommand = tauri::api::process::Command::new_sidecar("ffmpeg")
                    .expect("failed to create `ffmpeg` binary command")
                    .args([
                        "-i",
                        &self.microphone_output_path,
                        "-i",
                        &self.speakers_output_path,
                        "-filter_complex",
                        " amix=inputs=2:duration=first:dropout_transition=3",
                        "-b:a",
                        "128k",
                        &output_path,
                    ])
                    .output()
                    .expect("failed to execute process");
                info!("[FFMPG] status: {:?}", ffmpegcommand.status);
                info!("[FFMPG] stdout: {:?}", String::from(&ffmpegcommand.stdout));
                if !ffmpegcommand.status.success() {
                    error!("[FFMPG] stderr: {:?}", String::from(&ffmpegcommand.stderr));
                    panic!("FFMPEG failed to merge the audio files");
                }
                info!("[FFMPG] COMPLETED - {}", output_path);

                let meeting = Meeting::new(
                    Some(self.uuid),
                    self.starting_time.to_rfc2822(),
                    self.starting_time,
                    output_path,
                );
                return Ok(meeting);
            }
            State::Stopped => return Err(anyhow::anyhow!("Recording not started")),
        }
    }
}
