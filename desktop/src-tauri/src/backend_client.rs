use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    contracts::{ApiError, BackendInfo, ConnectionState, DeviceEvent, ErrorCode, PROTOCOL_VERSION},
    state::AppState,
};

const MAX_LINE_BYTES: usize = 64 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(12);
const SLOW_REQUEST_TIMEOUT: Duration = Duration::from_secs(45);
type PendingSender = mpsc::Sender<Result<Value, ApiError>>;

struct BackendProcess {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
}

struct ClientInner {
    process: Mutex<Option<BackendProcess>>,
    startup: Mutex<()>,
    pending: Mutex<HashMap<String, PendingSender>>,
    next_id: AtomicU64,
    healthy: AtomicBool,
    stopping: AtomicBool,
    app: Mutex<Option<AppHandle>>,
}

#[derive(Clone)]
pub struct BackendClient {
    inner: Arc<ClientInner>,
}

impl Default for BackendClient {
    fn default() -> Self {
        Self {
            inner: Arc::new(ClientInner {
                process: Mutex::new(None),
                startup: Mutex::new(()),
                pending: Mutex::new(HashMap::new()),
                next_id: AtomicU64::new(1),
                healthy: AtomicBool::new(false),
                stopping: AtomicBool::new(false),
                app: Mutex::new(None),
            }),
        }
    }
}

impl BackendClient {
    pub fn attach_app(&self, app: AppHandle) {
        *self.inner.app.lock().expect("app mutex poisoned") = Some(app);
    }

    pub fn request<P, R>(&self, method: &'static str, params: &P) -> Result<R, ApiError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        self.ensure_started()?;
        self.request_running(method, params)
    }

    fn request_running<P, R>(&self, method: &'static str, params: &P) -> Result<R, ApiError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let params = serde_json::to_value(params)
            .map_err(|_| ApiError::invalid("request could not be serialized"))?;
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed).to_string();
        let wire = serde_json::to_vec(&json!({
            "v": PROTOCOL_VERSION, "id": id, "method": method, "params": params
        }))
        .map_err(|_| ApiError::invalid("request could not be serialized"))?;
        if wire.len() + 1 > MAX_LINE_BYTES {
            return Err(ApiError::invalid("request is too large"));
        }

        let (sender, receiver) = mpsc::channel();
        self.inner
            .pending
            .lock()
            .map_err(|_| ApiError::internal("backend state is unavailable"))?
            .insert(id.clone(), sender);

        let write_result = (|| -> Result<(), ApiError> {
            let process = self.inner.process.lock().map_err(|_| ApiError::backend_unavailable())?;
            let process = process.as_ref().ok_or_else(ApiError::backend_unavailable)?;
            let mut stdin = process.stdin.lock().map_err(|_| ApiError::backend_unavailable())?;
            stdin.write_all(&wire).and_then(|_| stdin.write_all(b"\n")).and_then(|_| stdin.flush())
                .map_err(|_| ApiError::backend_crashed("The backend input stream closed"))
        })();
        if write_result.is_err() {
            self.inner.pending.lock().ok().and_then(|mut pending| pending.remove(&id));
            return Err(ApiError::backend_crashed("The backend input stream closed"));
        }

        let timeout = match method {
            "connect" | "discover" | "get_profiles" | "set_profile" | "delete_profile" => {
                SLOW_REQUEST_TIMEOUT
            }
            _ => REQUEST_TIMEOUT,
        };
        let value = match receiver.recv_timeout(timeout) {
            Ok(result) => result?,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.inner.pending.lock().ok().and_then(|mut pending| pending.remove(&id));
                return Err(ApiError::timeout());
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(ApiError::backend_crashed("The backend exited"));
            }
        };
        serde_json::from_value(value)
            .map_err(|_| ApiError::backend_crashed("The backend returned invalid data"))
    }

    fn ensure_started(&self) -> Result<(), ApiError> {
        let _startup = self.inner.startup.lock().map_err(|_| ApiError::backend_unavailable())?;
        {
            let process = self.inner.process.lock().map_err(|_| ApiError::backend_unavailable())?;
            if process.is_some() && self.inner.healthy.load(Ordering::Acquire) {
                return Ok(());
            }
        }
        self.stop_process();
        self.inner.stopping.store(false, Ordering::Release);
        let (child, stdin, stdout) = spawn_backend()?;
        let stdin = Arc::new(Mutex::new(stdin));
        self.inner.healthy.store(true, Ordering::Release);
        {
            let mut process = self.inner.process.lock().map_err(|_| ApiError::backend_unavailable())?;
            *process = Some(BackendProcess { child, stdin: stdin.clone() });
        }
        let inner = self.inner.clone();
        thread::Builder::new()
            .name("bosectl-backend-reader".into())
            .spawn(move || read_backend(inner, stdout))
            .map_err(|_| ApiError::backend_unavailable())?;

        let hello: BackendInfo = match self.request_running::<_, BackendInfo>("hello", &json!({})) {
            Ok(info) if info.protocol_version == PROTOCOL_VERSION => info,
            Ok(_) => {
                self.stop_process();
                return Err(ApiError::new(
                    ErrorCode::UnsupportedProtocol,
                    "The backend protocol version is not supported",
                    false,
                ));
            }
            Err(error) => {
                self.stop_process();
                return Err(error);
            }
        };
        if hello.backend_version.chars().count() > 64
            || hello.python_version.chars().count() > 64
            || !hello.operations.iter().any(|method| method == "shutdown")
        {
            self.stop_process();
            return Err(ApiError::backend_crashed("The backend handshake is incomplete"));
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        let _startup = match self.inner.startup.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        self.inner.stopping.store(true, Ordering::Release);
        if self.inner.healthy.load(Ordering::Acquire) {
            let _: Result<Value, ApiError> = self.request_running("shutdown", &json!({}));
        }
        self.stop_process();
    }

    fn stop_process(&self) {
        self.inner.stopping.store(true, Ordering::Release);
        self.inner.healthy.store(false, Ordering::Release);
        let process = self.inner.process.lock().ok().and_then(|mut process| process.take());
        if let Some(mut process) = process {
            drop(process.stdin);
            if process.child.try_wait().ok().flatten().is_none() {
                let _ = process.child.kill();
            }
            let _ = process.child.wait();
        }
        fail_pending(&self.inner, ApiError::backend_crashed("The backend stopped"));
    }
}

fn read_backend(inner: Arc<ClientInner>, stdout: impl std::io::Read) {
    let mut reader = BufReader::new(stdout);
    loop {
        let line = match read_capped_line(&mut reader) {
            Ok(None) => break,
            Ok(Some(line)) => line,
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                fail_all(&inner, ApiError::backend_crashed("The backend emitted an oversized message"));
                break;
            }
            Err(_) => break,
        };
        let message: Value = match serde_json::from_slice(&line) {
            Ok(message) => message,
            Err(_) => {
                fail_all(&inner, ApiError::backend_crashed("The backend emitted malformed JSON"));
                break;
            }
        };
        if message.get("v").and_then(Value::as_u64) != Some(PROTOCOL_VERSION.into()) {
            fail_all(&inner, ApiError::backend_crashed("The backend emitted an unsupported message"));
            break;
        }
        if let Some(id) = message.get("id").and_then(Value::as_str) {
            let sender = inner.pending.lock().ok().and_then(|mut pending| pending.remove(id));
            if let Some(sender) = sender {
                let response = if let Some(result) = message.get("result") {
                    Ok(result.clone())
                } else if let Some(error) = message.get("error") {
                    serde_json::from_value::<ApiError>(error.clone())
                        .map_or_else(|_| Err(ApiError::backend_crashed("The backend returned an invalid error")), Err)
                } else {
                    Err(ApiError::backend_crashed("The backend returned an invalid response"))
                };
                let _ = sender.send(response);
            }
            continue;
        }
        if message.get("event").is_some() {
            match serde_json::from_value::<DeviceEvent>(message) {
                Ok(event) if event.validate_wire().is_ok() => {
                    if let Some(app) = inner.app.lock().ok().and_then(|app| app.clone()) {
                        app.state::<AppState>().record_backend_event(&event);
                        update_tray_status(&app, &event);
                        let _ = app.emit_to("main", "bose://device-event", event);
                    }
                }
                _ => {
                    fail_all(&inner, ApiError::backend_crashed("The backend emitted an invalid event"));
                    break;
                }
            }
        }
    }
    inner.healthy.store(false, Ordering::Release);
    if !inner.stopping.load(Ordering::Acquire) {
        fail_all(&inner, ApiError::backend_crashed("The backend exited unexpectedly"));
    }
}

fn update_tray_status(app: &AppHandle, event: &DeviceEvent) {
    let tooltip = match event {
        DeviceEvent::SnapshotChanged { data }
            if data.connection.state == ConnectionState::Connected =>
        {
            let name = data.identity.as_ref().map_or("Headset", |identity| identity.name.as_str());
            let battery = data
                .battery
                .as_ref()
                .map(|battery| format!(" · {}%", battery.percent))
                .unwrap_or_default();
            format!("bosectl · {name}{battery}")
        }
        DeviceEvent::ConnectionChanged { data } => match data.state {
            ConnectionState::Discovering => "bosectl · Discovering…".into(),
            ConnectionState::Connecting | ConnectionState::Reconnecting => {
                "bosectl · Connecting…".into()
            }
            ConnectionState::Disconnecting => "bosectl · Disconnecting…".into(),
            ConnectionState::Disconnected => "bosectl · Disconnected".into(),
            ConnectionState::Error => "bosectl · Connection error".into(),
            ConnectionState::Connected => "bosectl · Connected".into(),
        },
        DeviceEvent::BackendError { .. } => "bosectl · Backend error".into(),
        DeviceEvent::SnapshotChanged { .. } => "bosectl".into(),
    };
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn read_capped_line<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok((!line.is_empty()).then_some(line));
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        if line.len() + take > MAX_LINE_BYTES {
            reader.consume(take);
            if newline.is_none() {
                discard_line(reader)?;
            }
            return Err(io::Error::new(io::ErrorKind::InvalidData, "oversized backend line"));
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if newline.is_some() {
            return Ok(Some(line));
        }
    }
}

fn discard_line<R: BufRead>(reader: &mut R) -> io::Result<()> {
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(());
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        reader.consume(take);
        if newline.is_some() {
            return Ok(());
        }
    }
}

fn fail_all(inner: &ClientInner, error: ApiError) {
    fail_pending(inner, error.clone());
    if let Some(app) = inner.app.lock().ok().and_then(|app| app.clone()) {
        let event = DeviceEvent::BackendError { data: error };
        app.state::<AppState>().record_backend_event(&event);
        update_tray_status(&app, &event);
        let _ = app.emit_to(
            "main",
            "bose://device-event",
            event,
        );
    }
}

fn fail_pending(inner: &ClientInner, error: ApiError) {
    if let Ok(mut pending) = inner.pending.lock() {
        for (_, sender) in pending.drain() {
            let _ = sender.send(Err(error.clone()));
        }
    }
}

#[cfg(debug_assertions)]
fn spawn_backend() -> Result<(Child, ChildStdin, std::process::ChildStdout), ApiError> {
    let python_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python");
    let mut command = if let Some(executable) = std::env::var_os("PYBMAP_PYTHON") {
        Command::new(executable)
    } else {
        Command::new("python")
    };
    command.args(["-m", "pybmap.backend"]).current_dir(python_dir);
    spawn_command(command)
}

#[cfg(not(debug_assertions))]
fn spawn_backend() -> Result<(Child, ChildStdin, std::process::ChildStdout), ApiError> {
    let executable = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("bosectl-backend.exe")))
        .ok_or_else(ApiError::backend_unavailable)?;
    spawn_command(Command::new(executable))
}

fn spawn_command(mut command: Command) -> Result<(Child, ChildStdin, std::process::ChildStdout), ApiError> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| ApiError::backend_unavailable())?;
    let stdin = child.stdin.take().ok_or_else(ApiError::backend_unavailable)?;
    let stdout = child.stdout.take().ok_or_else(ApiError::backend_unavailable)?;
    Ok((child, stdin, stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_output_fails_pending_requests() {
        let client = BackendClient::default();
        let (sender, receiver) = mpsc::channel();
        client.inner.pending.lock().unwrap().insert("1".into(), sender);
        read_backend(client.inner.clone(), b"not-json\n".as_slice());
        assert_eq!(receiver.recv().unwrap().unwrap_err().code, ErrorCode::BackendCrashed);
    }

    #[test]
    fn oversized_output_fails_pending_requests() {
        let client = BackendClient::default();
        let (sender, receiver) = mpsc::channel();
        client.inner.pending.lock().unwrap().insert("1".into(), sender);
        read_backend(client.inner.clone(), vec![b'x'; MAX_LINE_BYTES + 1].as_slice());
        assert_eq!(receiver.recv().unwrap().unwrap_err().code, ErrorCode::BackendCrashed);
    }

    #[test]
    fn responses_are_correlated_even_when_they_arrive_out_of_order() {
        let client = BackendClient::default();
        let (first_sender, first_receiver) = mpsc::channel();
        let (second_sender, second_receiver) = mpsc::channel();
        {
            let mut pending = client.inner.pending.lock().unwrap();
            pending.insert("first".into(), first_sender);
            pending.insert("second".into(), second_sender);
        }
        let output = br#"{"v":3,"id":"second","result":{"value":2}}
{"v":3,"id":"first","result":{"value":1}}
"#;
        read_backend(client.inner.clone(), output.as_slice());
        assert_eq!(first_receiver.recv().unwrap().unwrap()["value"], 1);
        assert_eq!(second_receiver.recv().unwrap().unwrap()["value"], 2);
    }

    #[test]
    fn unexpected_eof_is_reported_as_a_backend_crash() {
        let client = BackendClient::default();
        let (sender, receiver) = mpsc::channel();
        client.inner.pending.lock().unwrap().insert("1".into(), sender);
        read_backend(client.inner.clone(), b"".as_slice());
        assert_eq!(receiver.recv().unwrap().unwrap_err().code, ErrorCode::BackendCrashed);
    }
}
