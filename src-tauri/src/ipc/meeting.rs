/* 
    Tauri IPC commands to bridge Project Frontend 
    Model Controller to Backend Model Controller
    Async logic is handled are
    Sync logic is handled in the model controller
*/
use super::{DeleteParams, GetParams, IpcResponse, UpdateParams, ModelMutateResultData };
use crate::MeetingControllerState;
use crate::model::{Meeting, MeetingForUpdate, MeetingsRef};
use log::info;
use tauri::command;

#[command]
pub fn get_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<Meeting> {
    info!("get_meeting called");
    let meeting_controller = meeting_controller_state.0.lock().unwrap();   
    return IpcResponse::from(meeting_controller.get(params.id));
}   

#[command]
pub fn list_meetings(
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<Vec<MeetingsRef>> {
    info!("list_meetings called");
    let meeting_controller = meeting_controller_state.0.lock().unwrap();
    let meetings = meeting_controller.list();
    return IpcResponse::from(meetings);
}

#[command]
pub fn update_meeting(
    params: UpdateParams<MeetingForUpdate>, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("update_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.update(params.data.meeting));
}

#[command]
pub fn delete_meeting(
    params: DeleteParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("delete_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.delete(params.id));
}

#[command]
pub fn delete_all_meeting(
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<()> {
    info!("delete_all_meetings called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.delete_all());
}

#[command]
pub fn archive_meeting(
    params: DeleteParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("archive_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.delete(params.id));
}

#[command]
pub fn export_all_meeting(
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<String> {
    info!("export called");
    let meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.export_all());
} 

#[command]
pub fn increment_async_ops_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("increment_async_ops_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.increment_async_ops(params.id));
}

#[command]
pub fn decrement_async_ops_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("decrement_async_ops_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.decrement_async_ops(params.id));
}

#[command]
pub fn summarize_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("summarize_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap();
    return IpcResponse::from(meeting_controller.summarize_meeting(params.id));
}

#[command(async)]
pub fn async_summarize_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<'_, MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    //  Summarize without blocking the UI
    info!("summarize_meeting called");
    let mut meeting;
    {
        // Acquiring lock on meeting_controller_state"
        let mut meeting_controller: std::sync::MutexGuard<'_, crate::model::MeetingController> = meeting_controller_state.0.lock().unwrap();
        meeting = match meeting_controller.get(params.id.clone()) {
            Ok(m) => {
                m
            },
            Err(error) => {
                return IpcResponse::from(Err(error));
            }
        };
        let _ = meeting_controller.increment_async_ops(params.id.clone());
        // Releasing lock on meeting_controller_state"
    }
    // ? Summarizing can take a while
    let summary =  match meeting.summarize() {
        Ok(_) => {
            meeting.summary
        },
        Err(error) => {
            let mut meeting_controller: std::sync::MutexGuard<'_, crate::model::MeetingController> = meeting_controller_state.0.lock().unwrap();
            let _ = meeting_controller.decrement_async_ops(params.id.clone());
            return IpcResponse::from(Err(error));
        }
    };
    let mut meeting_controller: std::sync::MutexGuard<'_, crate::model::MeetingController> = meeting_controller_state.0.lock().unwrap();
    let mut meeting = match meeting_controller.get(params.id.clone()) {
        Ok(m) => {
            m
        },
        Err(error) => {
            let _ = meeting_controller.decrement_async_ops(params.id.clone());
            return IpcResponse::from(Err(error));
        }
    };
    meeting.summary = summary;
    let _ = meeting_controller.decrement_async_ops(params.id.clone());
    return IpcResponse::from(meeting_controller.update(meeting));
}

#[command]
pub fn improve_note_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    info!("improve_note_meeting called");
    let mut meeting_controller = meeting_controller_state.0.lock().unwrap(); 
    return IpcResponse::from(meeting_controller.improve_note_meeting(params.id));
}

#[command(async)]
pub fn async_improve_note_meeting(
    params: GetParams, 
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    // Improve note without blocking the UI
    info!("improve_note_meeting called");
    let mut meeting;
    {
        // Acquiring lock on meeting_controller_state"
        let mut meeting_controller: std::sync::MutexGuard<'_, crate::model::MeetingController> = meeting_controller_state.0.lock().unwrap();
        meeting = match meeting_controller.get(params.id.clone()) {
            Ok(m) => {
                m
            },
            Err(error) => {
                return IpcResponse::from(Err(error));
            }
        };
        let _ = meeting_controller.increment_async_ops(params.id.clone());
        // Releasing lock on meeting_controller_state"
    }
    // ? Summarizing can take a while
    let note =  match meeting.improve_note() {
        Ok(_) => {
            meeting.note
        },
        Err(error) => {
            let mut meeting_controller: std::sync::MutexGuard<'_, crate::model::MeetingController> = meeting_controller_state.0.lock().unwrap();
            let _ = meeting_controller.decrement_async_ops(params.id.clone());
            return IpcResponse::from(Err(error));
        }
    };

    // Acquiring lock on meeting_controller_state"
    let mut meeting_controller: std::sync::MutexGuard<'_, crate::model::MeetingController> = meeting_controller_state.0.lock().unwrap();
    let mut meeting = match meeting_controller.get(params.id.clone()) {
        Ok(m) => {
            m
        },
        Err(error) => {
           let _ = meeting_controller.decrement_async_ops(params.id.clone());
            return IpcResponse::from(Err(error));
        }
    };
    meeting.summary = note;
    let _ = meeting_controller.decrement_async_ops(params.id.clone());
    return IpcResponse::from(meeting_controller.update(meeting));
}
