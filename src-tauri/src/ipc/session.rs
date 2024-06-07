use log::info;
use tauri::command;
use crate::{SeesionState, utils::session::NewMeetingNote};
use super::{IpcResponse, UpdateParams};


#[command]
pub fn get_new_meeting_note(
    session_state: tauri::State<SeesionState>
) -> IpcResponse<NewMeetingNote> {
    info!("get_new_meeting_note called");
    let session_state = session_state.0.lock().unwrap();
    return IpcResponse::from(session_state.get_new_meeting_note()); 
}

#[command]
pub fn set_new_meeting_note(
    params: UpdateParams<NewMeetingNote>,
    session_state: tauri::State<SeesionState>
) -> IpcResponse<()> {
    info!("set_new_meeting_note called");
    let mut session_state = session_state.0.lock().unwrap();
    return IpcResponse::from(session_state.update_new_meeting_note(params.data));
}
