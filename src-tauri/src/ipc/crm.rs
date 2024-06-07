use log::{info, error, warn};
use serde_json::Value;
use super::{QueryParams, GetParams, IpcResponse, ModelMutateResultData};
use crate::{crm::affinity, MeetingControllerState};

#[tauri::command]
pub async fn search_organizations_crm(params: QueryParams) -> IpcResponse<Value> {
    let result: Result<Value, anyhow::Error> = affinity::async_search_organizations(&params.query, None, None).await;
    println!(": Result<Value, anyhow::Error>Hello from search_organizations");
    return IpcResponse::from(result);
}

#[tauri::command]
pub fn search_persons_crm(params: QueryParams) -> IpcResponse<Value> {
    let result: Result<Value, anyhow::Error> = affinity::get_persons(&params.query, None, None);
    return IpcResponse::from(result);
}

#[tauri::command]
pub fn get_organization_crm(params: GetParams) -> IpcResponse<Value> {
    let result = affinity::get_organizations(&params.id);
    return IpcResponse::from(result);
}

#[tauri::command]
pub fn publish_summary_crm(
    params: GetParams,
    meeting_controller_state: tauri::State<MeetingControllerState>
) -> IpcResponse<ModelMutateResultData> {
    let meeting_controller = meeting_controller_state.0.lock().unwrap();
    let meeting = meeting_controller.get(params.id);

    match meeting {
        Ok(meeting) => {
            let upload_response = IpcResponse::from(affinity::create_note(&meeting));
            match affinity::add_to_list(meeting) {
                Ok(result) => {
                    info!("[CRM] Successfully added meeting to list");
                }
                Err(error) => {
                    warn!("[CRM] Failed to add meeting to list: {}", error);
                }
            }
            return upload_response;            
        }
        Err(error) => {
            return IpcResponse::from(Err(error));
        }
    }
}

#[tauri::command]
pub fn list_lists_crm() -> IpcResponse<Value> {
    let result = affinity::get_lists();
    return IpcResponse::from(result);
}
