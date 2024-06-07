use super::{IpcResponse, UpdateParams};
use crate::SettingControllerState;
use tauri::command;
use crate::model::Setting;


#[command]
pub fn get_setting(
    setting_controller_state: tauri::State<SettingControllerState>
) -> IpcResponse<Setting> {
    let setting_controller = setting_controller_state.0.lock().unwrap();
    return IpcResponse::from(setting_controller.get_setting()); 
}


#[command]
pub fn update_setting(
    params: UpdateParams<Setting>,
    setting_controller_state: tauri::State<SettingControllerState>
) -> IpcResponse<()> {
    let mut setting_controller = setting_controller_state.0.lock().unwrap();
    return IpcResponse::from(setting_controller.set_setting(params.data));
}


#[command]
pub fn open_data_folder(
    setting_controller_state: tauri::State<SettingControllerState>
) -> IpcResponse<()> {
    let setting_controller = setting_controller_state.0.lock().unwrap();
    return IpcResponse::from(setting_controller.open_data_folder());
}