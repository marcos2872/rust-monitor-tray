use std::error::Error;

use tokio::sync::Mutex;
use zbus::{interface, ConnectionBuilder};

use crate::{
    cancel_network_speed_test, collect_fast_metrics_json, collect_metrics_json,
    collect_slow_metrics_json, get_network_speed_test_status_json,
    monitor::SystemMonitor, speedtest::NetworkSpeedTestManager, start_network_speed_test,
    DBUS_OBJECT_PATH, DBUS_SERVICE_NAME,
};

pub struct MetricsBackend {
    monitor: Mutex<SystemMonitor>,
    speed_test: NetworkSpeedTestManager,
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
            speed_test: NetworkSpeedTestManager::new(),
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

    async fn fast_metrics_json(&self) -> zbus::fdo::Result<String> {
        let mut monitor = self.monitor.lock().await;
        collect_fast_metrics_json(&mut monitor)
            .await
            .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
    }

    async fn slow_metrics_json(&self) -> zbus::fdo::Result<String> {
        let mut monitor = self.monitor.lock().await;
        collect_slow_metrics_json(&mut monitor)
            .await
            .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
    }

    async fn start_network_speed_test(&self) -> zbus::fdo::Result<bool> {
        Ok(start_network_speed_test(&self.speed_test).await)
    }

    async fn cancel_network_speed_test(&self) -> zbus::fdo::Result<bool> {
        Ok(cancel_network_speed_test(&self.speed_test).await)
    }

    async fn get_network_speed_test_status_json(&self) -> zbus::fdo::Result<String> {
        get_network_speed_test_status_json(&self.speed_test)
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
