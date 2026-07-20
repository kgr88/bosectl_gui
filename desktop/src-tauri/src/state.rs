use crate::backend_client::BackendClient;

#[derive(Default)]
pub struct AppState {
    pub backend: BackendClient,
}
