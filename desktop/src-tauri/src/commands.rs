use serde_json::json;
use tauri::State;

use crate::contracts::*;
use crate::state::AppState;

#[tauri::command]
pub fn get_backend_info(state: State<'_, AppState>) -> Result<BackendInfo, ApiError> {
    state.backend.request("hello", &json!({}))
}

#[tauri::command]
pub fn discover_devices(state: State<'_, AppState>) -> Result<DiscoverResult, ApiError> {
    state.backend.request("discover", &json!({}))
}

#[tauri::command]
pub fn connect_device(
    state: State<'_, AppState>,
    target: ConnectionTarget,
) -> Result<DeviceSnapshot, ApiError> {
    target.validate()?;
    state.backend.request("connect", &json!({ "target": target }))
}

#[tauri::command]
pub fn disconnect_device(state: State<'_, AppState>) -> Result<Ack, ApiError> {
    state.backend.request("disconnect", &json!({}))
}

#[tauri::command]
pub fn get_device_snapshot(state: State<'_, AppState>) -> Result<DeviceSnapshot, ApiError> {
    state.backend.request("get_snapshot", &json!({}))
}

#[tauri::command]
pub fn set_noise_control(
    state: State<'_, AppState>,
    request: SetNoiseControlRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.backend.request("set_noise_control", &request)
}

#[tauri::command]
pub fn set_equalizer(
    state: State<'_, AppState>,
    request: SetEqRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.backend.request("set_eq", &request)
}

#[tauri::command]
pub fn set_device_setting(
    state: State<'_, AppState>,
    request: SetSettingRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.backend.request("set_setting", &request)
}

#[tauri::command]
pub fn get_profiles(
    state: State<'_, AppState>,
    request: GenerationRequest,
) -> Result<ProfileList, ApiError> {
    state.backend.request("get_profiles", &request)
}

#[tauri::command]
pub fn set_profile(
    state: State<'_, AppState>,
    request: SetProfileRequest,
) -> Result<ProfileList, ApiError> {
    request.profile.validate()?;
    state.backend.request("set_profile", &request)
}

#[tauri::command]
pub fn delete_profile(
    state: State<'_, AppState>,
    request: DeleteProfileRequest,
) -> Result<ProfileList, ApiError> {
    if request.profile_id.is_empty() || request.profile_id.chars().count() > 256 {
        return Err(ApiError::invalid("profile_id is invalid"));
    }
    state.backend.request("delete_profile", &request)
}

#[tauri::command]
pub fn enter_pairing(
    state: State<'_, AppState>,
    request: GenerationRequest,
) -> Result<DeviceSnapshot, ApiError> {
    state.backend.request("enter_pairing", &request)
}

#[tauri::command]
pub fn power_off(
    state: State<'_, AppState>,
    request: GenerationRequest,
) -> Result<DeviceSnapshot, ApiError> {
    state.backend.request("power_off", &request)
}
