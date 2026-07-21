use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const PROTOCOL_VERSION: u8 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disconnected,
    Discovering,
    Connecting,
    Connected,
    Reconnecting,
    Disconnecting,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoiseControlKind {
    Cnc,
    Anr,
    Mode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SettingKey {
    Sidetone,
    AutoPause,
    SpatialAudio,
    WindBlock,
    AncToggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRequest,
    RequestTooLarge,
    UnsupportedProtocol,
    UnknownMethod,
    InvalidParams,
    BackendUnavailable,
    BackendCrashed,
    BackendTimeout,
    BluetoothUnavailable,
    BluetoothDisabled,
    DeviceNotFound,
    DeviceNotPaired,
    ServiceNotFound,
    AccessDenied,
    ConnectionFailed,
    NotConnected,
    DeviceTimeout,
    DeviceLost,
    UnsupportedDevice,
    UnsupportedOperation,
    StaleGeneration,
    Conflict,
    ShuttingDown,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self { code, message: message.into(), retryable, details: None }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidParams,
            message: message.into(),
            retryable: false,
            details: None,
        }
    }

    pub fn backend_unavailable() -> Self {
        Self::new(ErrorCode::BackendUnavailable, "The device backend is unavailable", true)
    }

    pub fn backend_crashed(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BackendCrashed, message, true)
    }

    pub fn timeout() -> Self {
        Self::new(ErrorCode::BackendTimeout, "The backend request timed out", true)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message, false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackendInfo {
    pub backend_version: String,
    pub python_version: String,
    pub protocol_version: u8,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub name: String,
    pub model: String,
    pub product_id: Option<u16>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatteryStatus {
    pub percent: u8,
    pub charging: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirmwareInfo {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionStatus {
    pub state: ConnectionState,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangeCapability {
    pub minimum: i16,
    pub maximum: i16,
    pub step: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoiseControlCapability {
    pub kind: NoiseControlKind,
    pub range: Option<RangeCapability>,
    pub options: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EqBandCapability {
    pub id: String,
    pub label: String,
    pub minimum: i16,
    pub maximum: i16,
    pub step: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettingControlKind {
    Toggle,
    Choice,
    Range,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettingCapability {
    pub key: SettingKey,
    pub label: String,
    pub kind: SettingControlKind,
    pub writable: bool,
    pub options: Vec<Choice>,
    pub minimum: Option<i16>,
    pub maximum: Option<i16>,
    pub step: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileCapability {
    pub maximum: u8,
    pub supports_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub noise_control: Option<NoiseControlCapability>,
    pub equalizer: Vec<EqBandCapability>,
    pub settings: Vec<SettingCapability>,
    pub profiles: Option<ProfileCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoiseControlValue {
    pub kind: NoiseControlKind,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceValues {
    pub noise_control: Option<NoiseControlValue>,
    pub eq: BTreeMap<String, i16>,
    pub settings: BTreeMap<SettingKey, Value>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceSnapshot {
    pub identity: Option<DeviceIdentity>,
    pub battery: Option<BatteryStatus>,
    pub firmware: Option<FirmwareInfo>,
    pub connection: ConnectionStatus,
    pub capabilities: DeviceCapabilities,
    pub values: DeviceValues,
    pub generation: u64,
    pub revision: u64,
}

impl DeviceSnapshot {
    pub fn validate_wire(&self) -> Result<(), ApiError> {
        if self.battery.as_ref().is_some_and(|battery| battery.percent > 100)
            || self.identity.as_ref().is_some_and(|identity| {
                !valid_text(&identity.device_id, 256)
                    || !valid_text(&identity.name, 256)
                    || !valid_text(&identity.model, 256)
                    || identity.address.as_ref().is_some_and(|address| !valid_bluetooth_address(address))
            })
            || self.capabilities.equalizer.iter().any(|band| band.minimum > band.maximum || band.step == 0)
            || self.capabilities.settings.iter().any(|setting| {
                setting.step == Some(0)
                    || setting.minimum.zip(setting.maximum).is_some_and(|(minimum, maximum)| minimum > maximum)
            })
            || self.values.noise_control.as_ref().is_some_and(|noise| !is_scalar(&noise.value))
            || self.values.settings.values().any(|value| !is_scalar(value))
        {
            Err(ApiError::backend_crashed("The backend returned an invalid device snapshot"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredDevice {
    pub device_id: String,
    pub address: Option<String>,
    pub name: String,
    pub product_id: Option<u16>,
    pub device_type: Option<String>,
    pub rfcomm_channel: Option<u8>,
    pub paired: bool,
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionTarget {
    Discovered { device_id: String },
    Manual { address: String, device_type: String },
}

impl ConnectionTarget {
    pub fn validate(&self) -> Result<(), ApiError> {
        match self {
            Self::Discovered { device_id } if valid_text(device_id, 256) => Ok(()),
            Self::Manual { address, device_type }
                if valid_bluetooth_address(address) && matches!(device_type.as_str(), "qc_ultra2" | "qc35") => Ok(()),
            Self::Discovered { .. } => Err(ApiError::invalid("device_id is invalid")),
            Self::Manual { .. } => Err(ApiError::invalid("manual target is invalid")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NoiseControlInput {
    Integer(i16),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetNoiseControlRequest {
    pub generation: u64,
    pub kind: NoiseControlKind,
    pub value: NoiseControlInput,
}

impl SetNoiseControlRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        match (&self.kind, &self.value) {
            (NoiseControlKind::Cnc, NoiseControlInput::Integer(value)) if (0..=10).contains(value) => Ok(()),
            (NoiseControlKind::Anr | NoiseControlKind::Mode, NoiseControlInput::Text(value))
                if valid_text(value, 64) => Ok(()),
            _ => Err(ApiError::invalid("noise-control kind and value do not match")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetEqRequest {
    pub generation: u64,
    pub bass: i16,
    pub mid: i16,
    pub treble: i16,
}

impl SetEqRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        if [self.bass, self.mid, self.treble]
            .iter()
            .all(|value| (-10..=10).contains(value))
        {
            Ok(())
        } else {
            Err(ApiError::invalid("EQ bands must be between -10 and 10"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetSettingRequest {
    pub generation: u64,
    pub key: SettingKey,
    pub value: Value,
}

impl SetSettingRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        let valid = match self.key {
            SettingKey::AutoPause | SettingKey::WindBlock | SettingKey::AncToggle => self.value.is_boolean(),
            SettingKey::Sidetone => matches!(self.value.as_str(), Some("off" | "low" | "medium" | "high")),
            SettingKey::SpatialAudio => matches!(self.value.as_str(), Some("off" | "room" | "motion")),
        };
        if valid {
            Ok(())
        } else {
            Err(ApiError::invalid("setting key and value do not match"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationRequest {
    pub generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileInput {
    pub profile_id: Option<String>,
    pub name: String,
    pub cnc_level: u8,
    pub auto_cnc: bool,
    pub spatial: String,
    pub wind_block: bool,
    pub anc_toggle: bool,
}

impl ProfileInput {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.profile_id.as_ref().is_some_and(|id| !valid_text(id, 256))
            || !valid_text(&self.name, 64)
            || self.cnc_level > 10
            || !matches!(self.spatial.as_str(), "off" | "room" | "motion")
        {
            Err(ApiError::invalid("profile is invalid"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetProfileRequest {
    pub generation: u64,
    pub profile: ProfileInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteProfileRequest {
    pub generation: u64,
    pub profile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub profile_id: String,
    pub name: String,
    pub editable: bool,
    pub cnc_level: u8,
    pub auto_cnc: bool,
    pub spatial: String,
    pub wind_block: bool,
    pub anc_toggle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ack {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoverResult {
    pub devices: Vec<DiscoveredDevice>,
}

impl DiscoverResult {
    pub fn validate_wire(&self) -> Result<(), ApiError> {
        if self.devices.iter().any(|device| {
            !valid_text(&device.device_id, 256)
                || !valid_text(&device.name, 256)
                || device.address.as_ref().is_some_and(|address| !valid_bluetooth_address(address))
                || device.device_type.as_ref().is_some_and(|kind| !matches!(kind.as_str(), "qc_ultra2" | "qc35"))
        }) {
            Err(ApiError::backend_crashed("The backend returned invalid discovery data"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileList {
    pub generation: u64,
    pub revision: u64,
    pub profiles: Vec<Profile>,
}

impl ProfileList {
    pub fn validate_wire(&self) -> Result<(), ApiError> {
        if self.profiles.iter().any(|profile| {
            !valid_text(&profile.profile_id, 256) || !valid_text(&profile.name, 64)
                || profile.cnc_level > 10 || !matches!(profile.spatial.as_str(), "off" | "room" | "motion")
        }) {
            Err(ApiError::backend_crashed("The backend returned invalid profile data"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DeviceEvent {
    SnapshotChanged { data: DeviceSnapshot },
    ConnectionChanged { data: ConnectionChangedData },
    BackendError { data: ApiError },
}

impl DeviceEvent {
    pub fn validate_wire(&self) -> Result<(), ApiError> {
        match self {
            Self::SnapshotChanged { data } => data.validate_wire(),
            Self::ConnectionChanged { data } if data.reason.as_ref().is_some_and(|reason| reason.chars().count() > 256) => {
                Err(ApiError::backend_crashed("The backend returned an invalid connection event"))
            }
            Self::BackendError { data } if data.message.chars().count() > 1024 => {
                Err(ApiError::backend_crashed("The backend returned an invalid error event"))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionChangedData {
    pub state: ConnectionState,
    pub generation: u64,
    pub revision: u64,
    pub reason: Option<String>,
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.chars().count() <= maximum
}

fn valid_bluetooth_address(value: &str) -> bool {
    let parts: Vec<_> = value.split(':').collect();
    parts.len() == 6 && parts.iter().all(|part| {
        part.len() == 2 && part.chars().all(|character| character.is_ascii_hexdigit())
    })
}

fn is_scalar(value: &Value) -> bool {
    value.is_boolean() || value.is_string() || value.is_number()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_stable_screaming_snake_case() {
        assert_eq!(serde_json::to_string(&ErrorCode::DeviceTimeout).unwrap(), "\"DEVICE_TIMEOUT\"");
    }

    #[test]
    fn manual_targets_reject_unknown_device_types() {
        let target = ConnectionTarget::Manual {
            address: "AA:BB:CC:DD:EE:FF".into(),
            device_type: "unknown".into(),
        };
        assert_eq!(target.validate().unwrap_err().code, ErrorCode::InvalidParams);
    }

    #[test]
    fn noise_control_value_must_match_kind() {
        let request = SetNoiseControlRequest {
            generation: 1,
            kind: NoiseControlKind::Cnc,
            value: NoiseControlInput::Text("high".into()),
        };
        assert!(request.validate().is_err());
    }

}
