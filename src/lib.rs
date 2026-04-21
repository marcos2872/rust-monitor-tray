pub mod dbus;
pub mod monitor;

use std::error::Error;

use monitor::{SystemMetrics, SystemMonitor};

pub const DBUS_SERVICE_NAME: &str = "com.monitortray.Backend";
pub const DBUS_OBJECT_PATH: &str = "/com/monitortray/Backend";
pub const DBUS_INTERFACE_NAME: &str = "com.monitortray.Backend";

pub async fn collect_metrics(monitor: &mut SystemMonitor) -> SystemMetrics {
    monitor.update_metrics().await;
    monitor.get_all_metrics()
}

pub async fn collect_metrics_once() -> SystemMetrics {
    let mut monitor = SystemMonitor::new();
    collect_metrics(&mut monitor).await
}

pub async fn collect_metrics_json(monitor: &mut SystemMonitor) -> Result<String, serde_json::Error> {
    serde_json::to_string(&collect_metrics(monitor).await)
}

pub async fn collect_metrics_once_json() -> Result<String, Box<dyn Error>> {
    let mut monitor = SystemMonitor::new();
    Ok(collect_metrics_json(&mut monitor).await?)
}
