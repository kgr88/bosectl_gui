use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tauri::State;

use crate::{backend_client::BackendClient, contracts::*, state::AppState};

async fn call<P, R>(backend: BackendClient, method: &'static str, params: P) -> Result<R, ApiError>
where
    P: Serialize + Send + 'static,
    R: DeserializeOwned + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || backend.request(method, &params))
        .await
        .map_err(|_| ApiError::internal("The backend task could not be joined"))?
}

#[tauri::command]
pub async fn get_backend_info(state: State<'_, AppState>) -> Result<BackendInfo, ApiError> {
    call(state.backend.clone(), "hello", json!({})).await
}

#[tauri::command]
pub async fn discover_devices(state: State<'_, AppState>) -> Result<DiscoverResult, ApiError> {
    let result: DiscoverResult = call(state.backend.clone(), "discover", json!({})).await?;
    result.validate_wire()?;
    Ok(result)
}

#[tauri::command]
pub async fn connect_device(
    state: State<'_, AppState>,
    target: ConnectionTarget,
) -> Result<DeviceSnapshot, ApiError> {
    target.validate()?;
    let attempt = state.begin_connection();
    let snapshot = call(
        state.backend.clone(),
        "connect",
        json!({ "target": target }),
    )
    .await?;
    state.accept_connection_snapshot(snapshot, attempt)
}

#[tauri::command]
pub async fn disconnect_device(state: State<'_, AppState>) -> Result<Ack, ApiError> {
    let result = call(state.backend.clone(), "disconnect", json!({})).await;
    state.backend.shutdown();
    state.clear_snapshot();
    result
}

#[tauri::command]
pub async fn get_device_snapshot(state: State<'_, AppState>) -> Result<DeviceSnapshot, ApiError> {
    let snapshot = call(state.backend.clone(), "get_snapshot", json!({})).await?;
    state.accept_snapshot(snapshot)
}

#[tauri::command]
pub fn get_current_device_snapshot(
    state: State<'_, AppState>,
) -> Result<Option<DeviceSnapshot>, ApiError> {
    state.current_snapshot()
}

#[tauri::command]
pub async fn poll_device_snapshot(
    state: State<'_, AppState>,
    request: PollSnapshotRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.validate_generation(request.generation)?;
    let snapshot = call(state.backend.clone(), "poll_snapshot", request).await?;
    state.accept_snapshot(snapshot)
}

#[tauri::command]
pub async fn set_noise_control(
    state: State<'_, AppState>,
    request: SetNoiseControlRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.validate_generation(request.generation)?;
    let snapshot = call(state.backend.clone(), "set_noise_control", request).await?;
    state.accept_snapshot(snapshot)
}

#[tauri::command]
pub async fn set_equalizer(
    state: State<'_, AppState>,
    request: SetEqRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.validate_generation(request.generation)?;
    let snapshot = call(state.backend.clone(), "set_eq", request).await?;
    state.accept_snapshot(snapshot)
}

#[tauri::command]
pub async fn set_device_setting(
    state: State<'_, AppState>,
    request: SetSettingRequest,
) -> Result<DeviceSnapshot, ApiError> {
    request.validate()?;
    state.validate_generation(request.generation)?;
    let snapshot = call(state.backend.clone(), "set_setting", request).await?;
    state.accept_snapshot(snapshot)
}

#[tauri::command]
pub async fn get_profiles(
    state: State<'_, AppState>,
    request: GenerationRequest,
) -> Result<ProfileList, ApiError> {
    state.validate_generation(request.generation)?;
    let result: ProfileList = call(state.backend.clone(), "get_profiles", request).await?;
    state.validate_generation(result.generation)?;
    result.validate_wire()?;
    Ok(result)
}

#[tauri::command]
pub async fn set_profile(
    state: State<'_, AppState>,
    request: SetProfileRequest,
) -> Result<ProfileList, ApiError> {
    request.profile.validate()?;
    state.validate_generation(request.generation)?;
    let result: ProfileList = call(state.backend.clone(), "set_profile", request).await?;
    state.validate_generation(result.generation)?;
    result.validate_wire()?;
    Ok(result)
}

#[tauri::command]
pub async fn delete_profile(
    state: State<'_, AppState>,
    request: DeleteProfileRequest,
) -> Result<ProfileList, ApiError> {
    if request.profile_id.is_empty() || request.profile_id.chars().count() > 256 {
        return Err(ApiError::invalid("profile_id is invalid"));
    }
    state.validate_generation(request.generation)?;
    let result: ProfileList = call(state.backend.clone(), "delete_profile", request).await?;
    state.validate_generation(result.generation)?;
    result.validate_wire()?;
    Ok(result)
}
