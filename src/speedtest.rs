use std::io;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::{Mutex, Notify};

use crate::monitor::{NetworkSpeedTestPhase, NetworkSpeedTestState, NetworkSpeedTestStatus};

const SPEED_TEST_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Clone)]
pub struct NetworkSpeedTestManager {
    inner: Arc<Inner>,
}

struct Inner {
    status: Mutex<NetworkSpeedTestStatus>,
    running: AtomicBool,
    cancel_notify: Notify,
}

struct ToolSpec {
    command: &'static str,
    args: &'static [&'static str],
    kind: ToolKind,
}

#[derive(Clone, Copy)]
enum ToolKind {
    Ookla,
    PythonCli,
}

struct ParsedSpeedTest {
    tool: String,
    ping_ms: f32,
    download_mbps: f32,
    upload_mbps: f32,
    server_name: Option<String>,
    server_location: Option<String>,
}

enum SpeedTestExecutionError {
    ToolNotFound,
    Failed(String),
    Cancelled,
    TimedOut,
}

impl Default for NetworkSpeedTestManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkSpeedTestManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                status: Mutex::new(NetworkSpeedTestStatus::default()),
                running: AtomicBool::new(false),
                cancel_notify: Notify::new(),
            }),
        }
    }

    pub async fn start(&self) -> bool {
        if self.inner.running.swap(true, Ordering::AcqRel) {
            return false;
        }

        self.set_status(NetworkSpeedTestStatus {
            state: NetworkSpeedTestState::Running,
            phase: NetworkSpeedTestPhase::Preparing,
            started_at_unix_ms: Some(current_unix_ms()),
            ..NetworkSpeedTestStatus::default()
        })
        .await;

        let manager = self.clone();
        tokio::spawn(async move {
            manager.run_in_background().await;
        });

        true
    }

    pub async fn cancel(&self) -> bool {
        if !self.inner.running.load(Ordering::Acquire) {
            return false;
        }

        self.inner.cancel_notify.notify_waiters();
        true
    }

    pub async fn get_status(&self) -> NetworkSpeedTestStatus {
        self.inner.status.lock().await.clone()
    }

    async fn set_status(&self, status: NetworkSpeedTestStatus) {
        *self.inner.status.lock().await = status;
    }

    async fn update_phase(&self, phase: NetworkSpeedTestPhase, tool: Option<&str>) {
        let mut status = self.inner.status.lock().await;
        status.phase = phase;
        if let Some(tool_name) = tool {
            status.tool = Some(tool_name.to_string());
        }
    }

    async fn run_in_background(&self) {
        let result = self.run_first_available_tool().await;
        match result {
            Ok(parsed) => {
                self.set_status(NetworkSpeedTestStatus {
                    state: NetworkSpeedTestState::Success,
                    phase: NetworkSpeedTestPhase::Done,
                    tool: Some(parsed.tool),
                    ping_ms: Some(parsed.ping_ms),
                    download_mbps: Some(parsed.download_mbps),
                    upload_mbps: Some(parsed.upload_mbps),
                    server_name: parsed.server_name,
                    server_location: parsed.server_location,
                    started_at_unix_ms: self.get_status().await.started_at_unix_ms,
                    finished_at_unix_ms: Some(current_unix_ms()),
                    error: None,
                })
                .await;
            }
            Err(SpeedTestExecutionError::Cancelled) => {
                self.set_status(NetworkSpeedTestStatus {
                    state: NetworkSpeedTestState::Cancelled,
                    phase: NetworkSpeedTestPhase::Cancelled,
                    started_at_unix_ms: self.get_status().await.started_at_unix_ms,
                    finished_at_unix_ms: Some(current_unix_ms()),
                    error: Some("Teste cancelado pelo usuário".to_string()),
                    ..NetworkSpeedTestStatus::default()
                })
                .await;
            }
            Err(SpeedTestExecutionError::TimedOut) => {
                self.set_status(NetworkSpeedTestStatus {
                    state: NetworkSpeedTestState::Error,
                    phase: NetworkSpeedTestPhase::Done,
                    started_at_unix_ms: self.get_status().await.started_at_unix_ms,
                    finished_at_unix_ms: Some(current_unix_ms()),
                    error: Some(
                        "Tempo limite excedido ao executar o teste de velocidade".to_string(),
                    ),
                    ..NetworkSpeedTestStatus::default()
                })
                .await;
            }
            Err(SpeedTestExecutionError::Failed(message)) => {
                self.set_status(NetworkSpeedTestStatus {
                    state: NetworkSpeedTestState::Error,
                    phase: NetworkSpeedTestPhase::Done,
                    started_at_unix_ms: self.get_status().await.started_at_unix_ms,
                    finished_at_unix_ms: Some(current_unix_ms()),
                    error: Some(message),
                    ..NetworkSpeedTestStatus::default()
                })
                .await;
            }
            Err(SpeedTestExecutionError::ToolNotFound) => {
                self.set_status(NetworkSpeedTestStatus {
                    state: NetworkSpeedTestState::Error,
                    phase: NetworkSpeedTestPhase::Done,
                    started_at_unix_ms: self.get_status().await.started_at_unix_ms,
                    finished_at_unix_ms: Some(current_unix_ms()),
                    error: Some("Nenhuma ferramenta de speed test encontrada. Instale `speedtest` ou `speedtest-cli`.".to_string()),
                    ..NetworkSpeedTestStatus::default()
                })
                .await;
            }
        }

        self.inner.running.store(false, Ordering::Release);
    }

    async fn run_first_available_tool(&self) -> Result<ParsedSpeedTest, SpeedTestExecutionError> {
        let tools = [
            ToolSpec {
                command: "speedtest",
                args: &["--accept-license", "--accept-gdpr", "--format=json"],
                kind: ToolKind::Ookla,
            },
            ToolSpec {
                command: "speedtest-cli",
                args: &["--json"],
                kind: ToolKind::PythonCli,
            },
        ];

        for tool in tools {
            self.update_phase(NetworkSpeedTestPhase::Running, Some(tool.command))
                .await;

            match self.run_tool(&tool).await {
                Err(SpeedTestExecutionError::ToolNotFound) => continue,
                other => return other,
            }
        }

        Err(SpeedTestExecutionError::ToolNotFound)
    }

    async fn run_tool(&self, tool: &ToolSpec) -> Result<ParsedSpeedTest, SpeedTestExecutionError> {
        let mut child = Command::new(tool.command)
            .args(tool.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    SpeedTestExecutionError::ToolNotFound
                } else {
                    SpeedTestExecutionError::Failed(format!(
                        "Falha ao iniciar `{}`: {}",
                        tool.command, error
                    ))
                }
            })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            SpeedTestExecutionError::Failed("Falha ao capturar stdout do speed test".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            SpeedTestExecutionError::Failed("Falha ao capturar stderr do speed test".to_string())
        })?;

        let stdout_task = tokio::spawn(async move {
            let mut reader = stdout;
            let mut buffer = Vec::new();
            let _ = reader.read_to_end(&mut buffer).await;
            buffer
        });
        let stderr_task = tokio::spawn(async move {
            let mut reader = stderr;
            let mut buffer = Vec::new();
            let _ = reader.read_to_end(&mut buffer).await;
            buffer
        });

        let exit_status = tokio::select! {
            status = child.wait() => {
                status.map_err(|error| SpeedTestExecutionError::Failed(format!(
                    "Falha ao aguardar `{}`: {}",
                    tool.command, error
                )))?
            }
            _ = self.inner.cancel_notify.notified() => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(SpeedTestExecutionError::Cancelled);
            }
            _ = tokio::time::sleep(SPEED_TEST_TIMEOUT) => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(SpeedTestExecutionError::TimedOut);
            }
        };

        let stdout_bytes = stdout_task.await.unwrap_or_default();
        let stderr_bytes = stderr_task.await.unwrap_or_default();
        let stdout_text = String::from_utf8_lossy(&stdout_bytes).trim().to_string();
        let stderr_text = String::from_utf8_lossy(&stderr_bytes).trim().to_string();

        if !exit_status.success() {
            return Err(SpeedTestExecutionError::Failed(build_exit_error_message(
                tool.command,
                &stderr_text,
                &stdout_text,
            )));
        }

        self.update_phase(NetworkSpeedTestPhase::Parsing, Some(tool.command))
            .await;

        parse_speedtest_output(tool.kind, tool.command, &stdout_text)
            .map_err(SpeedTestExecutionError::Failed)
    }
}

fn parse_speedtest_output(
    kind: ToolKind,
    tool_name: &str,
    payload: &str,
) -> Result<ParsedSpeedTest, String> {
    let json: Value = serde_json::from_str(payload)
        .map_err(|error| format!("Falha ao interpretar JSON de `{tool_name}`: {error}"))?;

    match kind {
        ToolKind::Ookla => parse_ookla_output(tool_name, &json),
        ToolKind::PythonCli => parse_python_cli_output(tool_name, &json),
    }
}

fn parse_ookla_output(tool_name: &str, json: &Value) -> Result<ParsedSpeedTest, String> {
    let ping_ms = read_f64(&json["ping"]["latency"], "ping.latency")? as f32;
    let download_bandwidth = read_f64(&json["download"]["bandwidth"], "download.bandwidth")?;
    let upload_bandwidth = read_f64(&json["upload"]["bandwidth"], "upload.bandwidth")?;

    Ok(ParsedSpeedTest {
        tool: tool_name.to_string(),
        ping_ms,
        download_mbps: bytes_per_second_to_mbps(download_bandwidth),
        upload_mbps: bytes_per_second_to_mbps(upload_bandwidth),
        server_name: read_string(&json["server"]["name"]),
        server_location: join_optional_parts(&[
            read_string(&json["server"]["location"]),
            read_string(&json["server"]["country"]),
        ]),
    })
}

fn parse_python_cli_output(tool_name: &str, json: &Value) -> Result<ParsedSpeedTest, String> {
    let ping_ms = read_f64(&json["ping"], "ping")? as f32;
    let download_bits = read_f64(&json["download"], "download")?;
    let upload_bits = read_f64(&json["upload"], "upload")?;

    Ok(ParsedSpeedTest {
        tool: tool_name.to_string(),
        ping_ms,
        download_mbps: bits_per_second_to_mbps(download_bits),
        upload_mbps: bits_per_second_to_mbps(upload_bits),
        server_name: join_optional_parts(&[
            read_string(&json["server"]["sponsor"]),
            read_string(&json["server"]["name"]),
        ]),
        server_location: join_optional_parts(&[
            read_string(&json["server"]["country"]),
            read_string(&json["server"]["host"]),
        ]),
    })
}

fn read_f64(value: &Value, path: &str) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("Campo `{path}` ausente ou inválido no resultado do speed test"))
}

fn read_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn join_optional_parts(parts: &[Option<String>]) -> Option<String> {
    let joined: Vec<String> = parts.iter().flatten().cloned().collect();
    if joined.is_empty() {
        None
    } else {
        Some(joined.join(" · "))
    }
}

fn bytes_per_second_to_mbps(bytes_per_second: f64) -> f32 {
    ((bytes_per_second * 8.0) / 1_000_000.0) as f32
}

fn bits_per_second_to_mbps(bits_per_second: f64) -> f32 {
    (bits_per_second / 1_000_000.0) as f32
}

fn build_exit_error_message(command: &str, stderr_text: &str, stdout_text: &str) -> String {
    if !stderr_text.is_empty() {
        format!("`{command}` falhou: {}", truncate_for_error(stderr_text))
    } else if !stdout_text.is_empty() {
        format!("`{command}` falhou: {}", truncate_for_error(stdout_text))
    } else {
        format!("`{command}` terminou com erro sem mensagem adicional")
    }
}

fn truncate_for_error(text: &str) -> String {
    const LIMIT: usize = 240;
    let trimmed = text.trim();
    if trimmed.chars().count() <= LIMIT {
        trimmed.to_string()
    } else {
        let shortened: String = trimmed.chars().take(LIMIT).collect();
        format!("{shortened}...")
    }
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ookla_output_converts_bandwidth_to_mbps() {
        let payload = r#"{
            "ping": { "latency": 12.4 },
            "download": { "bandwidth": 25000000 },
            "upload": { "bandwidth": 12500000 },
            "server": { "name": "Meu Servidor", "location": "São Paulo", "country": "BR" }
        }"#;

        let result = parse_speedtest_output(ToolKind::Ookla, "speedtest", payload)
            .expect("ookla payload should parse");

        assert!((result.download_mbps - 200.0).abs() < 0.01);
        assert!((result.upload_mbps - 100.0).abs() < 0.01);
        assert_eq!(result.server_name.as_deref(), Some("Meu Servidor"));
        assert_eq!(result.server_location.as_deref(), Some("São Paulo · BR"));
    }

    #[test]
    fn test_parse_python_cli_output_uses_bits_per_second() {
        let payload = r#"{
            "ping": 9.5,
            "download": 85000000,
            "upload": 15000000,
            "server": {
                "sponsor": "Provedor X",
                "name": "Node 1",
                "country": "BR",
                "host": "node1.example.com"
            }
        }"#;

        let result = parse_speedtest_output(ToolKind::PythonCli, "speedtest-cli", payload)
            .expect("python cli payload should parse");

        assert!((result.download_mbps - 85.0).abs() < 0.01);
        assert!((result.upload_mbps - 15.0).abs() < 0.01);
        assert_eq!(result.server_name.as_deref(), Some("Provedor X · Node 1"));
        assert_eq!(
            result.server_location.as_deref(),
            Some("BR · node1.example.com")
        );
    }
}
