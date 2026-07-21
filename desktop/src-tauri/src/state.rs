use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::{
    backend_client::BackendClient,
    contracts::{ApiError, DeviceSnapshot, ErrorCode},
    lifecycle::SingleInstanceGuard,
};

pub struct AppState {
    pub backend: BackendClient,
    generation: AtomicU64,
    revision: AtomicU64,
    pub quitting: AtomicBool,
    pub shutdown_complete: AtomicBool,
    pub _single_instance: SingleInstanceGuard,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::SingleInstanceGuard;

    fn snapshot(generation: u64, revision: u64) -> DeviceSnapshot {
        serde_json::from_value(serde_json::json!({
            "identity": null, "battery": null, "firmware": null,
            "connection": {"state": "connected", "reason": null},
            "capabilities": {"noise_control": null, "equalizer": [], "settings": [], "profiles": null},
            "values": {"noise_control": null, "eq": {}, "settings": {}, "profile_id": null},
            "generation": generation, "revision": revision
        })).unwrap()
    }

    #[test]
    fn rejects_old_generations_and_revisions() {
        let state = AppState::new(SingleInstanceGuard::for_test());
        state.accept_snapshot(snapshot(2, 4)).unwrap();
        assert_eq!(state.accept_snapshot(snapshot(1, 99)).unwrap_err().code, ErrorCode::StaleGeneration);
        assert_eq!(state.accept_snapshot(snapshot(2, 3)).unwrap_err().code, ErrorCode::StaleGeneration);
        assert!(state.accept_snapshot(snapshot(3, 0)).is_ok());
    }
}

impl AppState {
    pub fn new(single_instance: SingleInstanceGuard) -> Self {
        Self {
            backend: BackendClient::default(),
            generation: AtomicU64::new(0),
            revision: AtomicU64::new(0),
            quitting: AtomicBool::new(false),
            shutdown_complete: AtomicBool::new(false),
            _single_instance: single_instance,
        }
    }

    pub fn validate_generation(&self, generation: u64) -> Result<(), ApiError> {
        let current = self.generation.load(Ordering::Acquire);
        if current != 0 && generation != current {
            Err(ApiError::new(
                ErrorCode::StaleGeneration,
                "The request belongs to an older device connection",
                false,
            ))
        } else {
            Ok(())
        }
    }

    pub fn accept_snapshot(&self, snapshot: DeviceSnapshot) -> Result<DeviceSnapshot, ApiError> {
        snapshot.validate_wire()?;
        let current = self.generation.load(Ordering::Acquire);
        let revision = self.revision.load(Ordering::Acquire);
        if snapshot.generation < current
            || (snapshot.generation == current && snapshot.revision < revision)
        {
            return Err(ApiError::new(
                ErrorCode::StaleGeneration,
                "The backend returned a stale device snapshot",
                false,
            ));
        }
        self.generation.store(snapshot.generation, Ordering::Release);
        self.revision.store(snapshot.revision, Ordering::Release);
        Ok(snapshot)
    }
}
