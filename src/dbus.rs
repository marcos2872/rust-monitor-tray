use std::error::Error;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use zbus::{interface, ConnectionBuilder};

use crate::{
    cancel_network_speed_test, collect_history_metrics_json, collect_metrics_json,
    collect_slow_metrics_json, get_network_speed_test_status_json, monitor::SystemMonitor,
    speedtest::NetworkSpeedTestManager, start_network_speed_test, DBUS_OBJECT_PATH,
    DBUS_SERVICE_NAME,
};

const METRICS_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1000);

pub struct MetricsBackend {
    monitor: Arc<Mutex<SystemMonitor>>,
    fast_metrics_cache: Arc<RwLock<String>>,
    history_metrics_cache: Arc<RwLock<String>>,
    speed_test: NetworkSpeedTestManager,
}

impl Default for MetricsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsBackend {
    pub fn new() -> Self {
        let monitor = SystemMonitor::new();
        let initial_fast_metrics =
            serde_json::to_string(&monitor.get_fast_metrics()).unwrap_or_else(|_| "{}".to_string());
        let initial_history_metrics =
            collect_history_metrics_json(&monitor).unwrap_or_else(|_| "{}".to_string());
        let monitor = Arc::new(Mutex::new(monitor));
        let fast_metrics_cache = Arc::new(RwLock::new(initial_fast_metrics));
        let history_metrics_cache = Arc::new(RwLock::new(initial_history_metrics));

        spawn_metrics_updater(
            monitor.clone(),
            fast_metrics_cache.clone(),
            history_metrics_cache.clone(),
        );

        Self {
            monitor,
            fast_metrics_cache,
            history_metrics_cache,
            speed_test: NetworkSpeedTestManager::new(),
        }
    }
}

async fn refresh_backend_caches(
    monitor: &Arc<Mutex<SystemMonitor>>,
    fast_metrics_cache: &Arc<RwLock<String>>,
    history_metrics_cache: &Arc<RwLock<String>>,
    force_slow_refresh: bool,
) {
    let mut locked_monitor = monitor.lock().await;
    locked_monitor.update_fast_metrics().await;
    locked_monitor
        .refresh_slow_metrics(force_slow_refresh)
        .await;

    let fast_metrics_json = serde_json::to_string(&locked_monitor.get_fast_metrics())
        .unwrap_or_else(|_| "{}".to_string());
    let history_metrics_json =
        collect_history_metrics_json(&locked_monitor).unwrap_or_else(|_| "{}".to_string());
    drop(locked_monitor);

    *fast_metrics_cache.write().await = fast_metrics_json;
    *history_metrics_cache.write().await = history_metrics_json;
}

fn spawn_metrics_updater(
    monitor: Arc<Mutex<SystemMonitor>>,
    fast_metrics_cache: Arc<RwLock<String>>,
    history_metrics_cache: Arc<RwLock<String>>,
) {
    tokio::spawn(async move {
        refresh_backend_caches(&monitor, &fast_metrics_cache, &history_metrics_cache, true).await;

        loop {
            tokio::time::sleep(METRICS_REFRESH_INTERVAL).await;
            refresh_backend_caches(&monitor, &fast_metrics_cache, &history_metrics_cache, false)
                .await;
        }
    });
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
        Ok(self.fast_metrics_cache.read().await.clone())
    }

    async fn slow_metrics_json(&self) -> zbus::fdo::Result<String> {
        let mut monitor = self.monitor.lock().await;
        let needs_warmup = monitor.cached_sensors.is_none()
            || monitor.cached_top_processes.is_none()
            || monitor.cached_gpus.is_empty();

        if needs_warmup {
            collect_slow_metrics_json(&mut monitor)
                .await
                .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
        } else {
            serde_json::to_string(&monitor.get_slow_metrics())
                .map_err(|err| zbus::fdo::Error::Failed(err.to_string()))
        }
    }

    async fn history_metrics_json(&self) -> zbus::fdo::Result<String> {
        Ok(self.history_metrics_cache.read().await.clone())
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
