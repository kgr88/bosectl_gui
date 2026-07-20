use serde::{de::DeserializeOwned, Serialize};

use crate::contracts::ApiError;

/// Stage 1 seam for the supervised, long-lived sidecar.
///
/// Stage 2 replaces the unavailable response with process supervision and JSONL
/// correlation; commands and renderer types do not need to change.
#[derive(Default)]
pub struct BackendClient;

impl BackendClient {
    pub fn request<P, R>(&self, _method: &'static str, params: &P) -> Result<R, ApiError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        serde_json::to_value(params)
            .map_err(|_| ApiError::invalid("request could not be serialized"))?;
        Err(ApiError::backend_unavailable())
    }
}
