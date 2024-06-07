
use crate::{RecordingState, MeetingControllerState};
use crate::audio::recorder::State;
use anyhow::Error;
use super::{IpcResponse, GetTranscriptParams, GetRecordingStartParams};
use crate::model::Meeting;
use log::info;
use tauri::Window;
use crate::audio::cpal_audio::cpal_utils;

#[tauri::command]
pub fn get_recording_state(state: tauri::State<RecordingState>) -> IpcResponse<State> {
    info!("get_recording_state called");
    let recorder_guard = state.0.lock().unwrap();
    let result: Result<State, Error> = Ok(recorder_guard.get_state());
    return IpcResponse::from(result);
}

#[tauri::command]
pub fn get_recording_device_names(state: tauri::State<RecordingState>) -> IpcResponse<cpal_utils::RecordingDevices> {
    info!("get_recording_device_names called");
    let recorder_guard = state.0.lock().unwrap();
    return IpcResponse::from(recorder_guard.get_device_names());
}


#[tauri::command]
pub fn start_recording(
    params: GetRecordingStartParams,
    state: tauri::State<RecordingState>, 
    window: Window
) -> IpcResponse<String> {
    info!("start_recording called");
    info!("recording_devices: {} - {}", params.recording_devices.input_device_name, params.recording_devices.output_device_name);
    let mut recorder_guard = state.0.lock().unwrap();
    return IpcResponse::from(recorder_guard.start(window, Some(params.recording_devices.input_device_name), Some(params.recording_devices.output_device_name)));
}

#[tauri::command]
pub fn pause_recording(state: tauri::State<RecordingState>) -> IpcResponse<()> {
    info!("pause_recording called");
    let mut recorder_guard = state.0.lock().unwrap();
    return IpcResponse::from(recorder_guard.pause());
}

#[tauri::command]
pub fn resume_recording(state: tauri::State<RecordingState>) -> IpcResponse<()> {
    info!("resume_recording called");
    let mut recorder_guard = state.0.lock().unwrap();
    return IpcResponse::from(recorder_guard.resume());
}

#[tauri::command]
pub fn stop_recording(
    recording_state: tauri::State<RecordingState>, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<Meeting> {
    info!("stop_recording called");
    let mut recorder_guard = recording_state.0.lock().unwrap();
    let meeting = recorder_guard.stop();
    match meeting {
        Ok(ref meeting) => {
            let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
            let result = meeting_controller.add(meeting.clone());
            return IpcResponse::from(result);
        }
        Err(error) => {
            return IpcResponse::from(Err(error));
        }
    };
}

#[tauri::command]
pub async fn transcribe_recording(
    params: GetTranscriptParams,
) -> IpcResponse<String> {
    /*
        Designed to simply return the transcript of a given recording.
        The caller need to associate the transcript with the recording.
        ? Simplify a lot the logic, other wise the function would need to bloc de meeting data structure.
     */
    info!("transcribe_recording called");
    let result = crate::audio::processor::get_transcript(&params.path, Some(params.language)).await;
    info!("transcribe_recording result: {:?}", result);
    return IpcResponse::from(result);
}

#[tauri::command]
pub async fn get_available_audio_devices(window: Window) -> IpcResponse<cpal_utils::AvailableDevices> {
    info!("get_available_audio_devices called");
    return IpcResponse::from(cpal_utils::get_available_audio_devices(window));
}
