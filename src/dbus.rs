use std::error::Error;

use tokio::sync::Mutex;
use zbus::{interface, ConnectionBuilder};

use crate::{collect_metrics_json, monitor::SystemMonitor, DBUS_OBJECT_PATH, DBUS_SERVICE_NAME};

pub struct MetricsBackend {
    monitor: Mutex<SystemMonitor>,
}

impl Default for MetricsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsBackend {
    pub fn new() -> Self {
        Self {
            monitor: Mutex::new(SystemMonitor::new()),
        }
    }
}

#[interface(name = "com.monitortray.Backend")]
impl MetricsBackend {
    async fn ping(&self) -> &str {
        "ok"
    }

    async fn get_metrics_json(&self) -> zbus::fdo::Result<String> {
        let mut monitor = self.monitor.lock().await;
        collect_metrics_json(&mut monitor)
            .await
            .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
    }
}

pub async fn run_dbus_service() -> Result<(), Box<dyn Error>> {
    let _connection = ConnectionBuilder::session()?
        .name(DBUS_SERVICE_NAME)?
        .serve_at(DBUS_OBJECT_PATH, MetricsBackend::new())?
        .build()
        .await?;

    std::future::pending::<()>().await;
    #[allow(unreachable_code)]
    Ok(())
}
