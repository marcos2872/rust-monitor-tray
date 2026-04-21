mod monitor;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::monitor::{SystemMetrics, SystemMonitor};

struct IconCache {
    last_cpu: Option<u32>,
    last_memory: Option<u32>,
    current_path: Option<String>,
}

struct MenuItems {
    cpu_root_item: gtk::MenuItem,
    cpu_usage_item: gtk::MenuItem,
    mem_root_item: gtk::MenuItem,
    mem_usage_item: gtk::MenuItem,
    disk_root_item: gtk::MenuItem,
    disk_usage_item: gtk::MenuItem,
    disk_total_item: gtk::MenuItem,
    disk_available_item: gtk::MenuItem,
    network_root_item: gtk::MenuItem,
    swap_item: Option<gtk::MenuItem>,
    uptime_root_item: gtk::MenuItem,
    uptime_item: gtk::MenuItem,
    load_avg_item: gtk::MenuItem,
    total_rx_item: gtk::MenuItem,
    total_tx_item: gtk::MenuItem,
}
static FILE_TOGGLE: AtomicBool = AtomicBool::new(false);
static ICON_DIR: OnceLock<PathBuf> = OnceLock::new();

impl IconCache {
    fn new() -> Self {
        IconCache {
            last_cpu: None,
            last_memory: None,
            current_path: None,
        }
    }

    fn get_icon_path(
        &mut self,
        stats: &SystemMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cpu_rounded = stats.cpu.usage_percent.round() as u32;
        let memory_rounded = stats.memory.usage_percent.round() as u32;

        let needs_update = self.last_cpu != Some(cpu_rounded)
            || self.last_memory != Some(memory_rounded)
            || self.current_path.is_none();

        if needs_update {
            let path = create_status_icon(stats)?;
            self.last_cpu = Some(cpu_rounded);
            self.last_memory = Some(memory_rounded);
            self.current_path = Some(path);
        }

        self.current_path
            .clone()
            .ok_or_else(|| "caminho do ícone indisponível".into())
    }
}

fn create_bar_chart(value: f32, max_value: f32, width: u32) -> String {
    let safe_value = if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    };
    let ratio = if max_value.is_finite() && max_value > 0.0 {
        (safe_value / max_value).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled_width = (ratio * width as f32).round() as u32;
    let empty_width = width.saturating_sub(filled_width);
    let indicator = usage_indicator(safe_value);
    let filled_bar = "█".repeat(filled_width as usize);
    let empty_bar = "░".repeat(empty_width as usize);

    format!(
        "{} {} {:>5.1}%",
        indicator,
        filled_bar + &empty_bar,
        safe_value
    )
}

fn format_bytes(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    } else if bytes >= 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else {
        format!("{:.0} B", bytes)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn icon_directory() -> Result<&'static PathBuf, Box<dyn std::error::Error>> {
    if let Some(dir) = ICON_DIR.get() {
        return Ok(dir);
    }

    let dir = if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join(format!("monitor-tray-{}", std::process::id()))
    } else {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        std::env::temp_dir().join(format!("monitor-tray-{}-{}", std::process::id(), timestamp))
    };

    std::fs::create_dir_all(&dir)?;
    let _ = ICON_DIR.set(dir);

    ICON_DIR
        .get()
        .ok_or_else(|| "falha ao inicializar diretório de ícones".into())
}

fn create_disabled_item(label: &str) -> gtk::MenuItem {
    let item = gtk::MenuItem::with_label(label);
    item.set_sensitive(false);
    item
}

fn create_submenu() -> gtk::Menu {
    let submenu = gtk::Menu::new();
    submenu.set_size_request(380, -1);
    submenu
}

fn create_submenu_item(label: &str, submenu: &gtk::Menu) -> gtk::MenuItem {
    let item = gtk::MenuItem::with_label(label);
    item.set_submenu(Some(submenu));
    item
}

fn append_separator(menu: &gtk::Menu) {
    menu.append(&gtk::SeparatorMenuItem::new());
}

fn usage_indicator(value: f32) -> &'static str {
    let usage = normalized_usage(value);
    if usage < 50.0 {
        "🟢"
    } else if usage < 80.0 {
        "🟡"
    } else {
        "🔴"
    }
}

fn padded_root_label(icon: &str, text: &str, value: &str) -> String {
    format!("   {icon} {text} {value}   ")
}

fn cpu_summary_label(stats: &SystemMetrics) -> String {
    padded_root_label(
        "🖥",
        "CPU",
        &format!("{:>4.0}%", normalized_usage(stats.cpu.usage_percent)),
    )
}

fn memory_summary_label(stats: &SystemMetrics) -> String {
    padded_root_label(
        "🧠",
        "RAM",
        &format!("{:>4.0}%", normalized_usage(stats.memory.usage_percent)),
    )
}

fn disk_usage_percent(stats: &SystemMetrics) -> f32 {
    if stats.disk.total_space > 0.0 {
        ((stats.disk.used_space / stats.disk.total_space) * 100.0) as f32
    } else {
        0.0
    }
}

fn disk_summary_label(stats: &SystemMetrics) -> String {
    padded_root_label(
        "💾",
        "Disco",
        &format!("{:>4.0}%", normalized_usage(disk_usage_percent(stats))),
    )
}

fn network_summary_label(stats: &SystemMetrics) -> String {
    padded_root_label(
        "🌐",
        "Rede",
        &format!("{} ifs", stats.network.interfaces.len()),
    )
}

fn system_summary_label(stats: &SystemMetrics) -> String {
    padded_root_label("⏱", "Sistema", &format_uptime(stats.uptime))
}

fn cpu_usage_label(stats: &SystemMetrics) -> String {
    format!(
        "Uso: {}",
        create_bar_chart(stats.cpu.usage_percent, 100.0, 12)
    )
}

fn memory_usage_label(stats: &SystemMetrics) -> String {
    format!(
        "RAM: {}",
        create_bar_chart(stats.memory.usage_percent, 100.0, 12)
    )
}

fn swap_label(stats: &SystemMetrics) -> Option<String> {
    if stats.memory.total_swap <= 0.0 {
        return None;
    }

    let swap_usage_percent = (stats.memory.used_swap / stats.memory.total_swap) * 100.0;
    Some(format!(
        "SWAP: {} ({:.1}/{:.1} GB)",
        create_bar_chart(swap_usage_percent as f32, 100.0, 12),
        stats.memory.used_swap,
        stats.memory.total_swap
    ))
}

fn disk_usage_label(stats: &SystemMetrics) -> String {
    format!(
        "Uso: {}",
        create_bar_chart(disk_usage_percent(stats), 100.0, 12)
    )
}

fn disk_total_label(stats: &SystemMetrics) -> String {
    format!("Total: {:.1} GB", stats.disk.total_space)
}

fn disk_available_label(stats: &SystemMetrics) -> String {
    format!("Disponível: {:.1} GB", stats.disk.available_space)
}

fn total_rx_label(stats: &SystemMetrics) -> String {
    format!(
        "Recebido: ↓{}",
        format_bytes(stats.network.total_bytes_received as f64)
    )
}

fn total_tx_label(stats: &SystemMetrics) -> String {
    format!(
        "Enviado: ↑{}",
        format_bytes(stats.network.total_bytes_transmitted as f64)
    )
}

fn load_average_label(stats: &SystemMetrics) -> String {
    format!(
        "Load avg: {:.2} / {:.2} / {:.2}",
        stats.load_average.0, stats.load_average.1, stats.load_average.2
    )
}

fn uptime_label(stats: &SystemMetrics) -> String {
    format!("Uptime: {}", format_uptime(stats.uptime))
}

fn append_cpu_section(menu: &gtk::Menu, stats: &SystemMetrics) -> (gtk::MenuItem, gtk::MenuItem) {
    let submenu = create_submenu();
    let cpu_root_item = create_submenu_item(&cpu_summary_label(stats), &submenu);
    menu.append(&cpu_root_item);

    submenu.append(&create_disabled_item("🖥 CPU"));
    submenu.append(&create_disabled_item(&format!(
        "Resumo: {:>5.1}%",
        normalized_usage(stats.cpu.usage_percent)
    )));

    let cpu_usage_item = create_disabled_item(&cpu_usage_label(stats));
    submenu.append(&cpu_usage_item);
    submenu.append(&create_disabled_item(&format!(
        "Modelo: {}",
        stats.cpu.name
    )));
    submenu.append(&create_disabled_item(&format!(
        "Núcleos: {}",
        stats.cpu.core_count
    )));
    submenu.append(&create_disabled_item(&format!(
        "Frequência: {} MHz",
        stats.cpu.frequency
    )));

    (cpu_root_item, cpu_usage_item)
}

fn append_memory_section(
    menu: &gtk::Menu,
    stats: &SystemMetrics,
) -> (gtk::MenuItem, gtk::MenuItem, Option<gtk::MenuItem>) {
    let submenu = create_submenu();
    let mem_root_item = create_submenu_item(&memory_summary_label(stats), &submenu);
    menu.append(&mem_root_item);

    submenu.append(&create_disabled_item("🧠 Memória"));
    submenu.append(&create_disabled_item(&format!(
        "Resumo: {:>5.1}%",
        normalized_usage(stats.memory.usage_percent)
    )));

    let mem_usage_item = create_disabled_item(&memory_usage_label(stats));
    submenu.append(&mem_usage_item);
    submenu.append(&create_disabled_item(&format!(
        "Usada: {:.1}/{:.1} GB",
        stats.memory.used_memory, stats.memory.total_memory
    )));
    submenu.append(&create_disabled_item(&format!(
        "Disponível: {:.1} GB",
        stats.memory.available_memory
    )));

    let swap_item = swap_label(stats).map(|label| {
        let item = create_disabled_item(&label);
        submenu.append(&item);
        item
    });

    (mem_root_item, mem_usage_item, swap_item)
}

fn append_disk_section(
    menu: &gtk::Menu,
    stats: &SystemMetrics,
) -> (gtk::MenuItem, gtk::MenuItem, gtk::MenuItem, gtk::MenuItem) {
    let submenu = create_submenu();
    let disk_root_item = create_submenu_item(&disk_summary_label(stats), &submenu);
    menu.append(&disk_root_item);

    submenu.append(&create_disabled_item("💾 Disco"));
    submenu.append(&create_disabled_item(&format!(
        "Resumo: {:>5.1}%",
        normalized_usage(disk_usage_percent(stats))
    )));

    let disk_usage_item = create_disabled_item(&disk_usage_label(stats));
    let disk_total_item = create_disabled_item(&disk_total_label(stats));
    let disk_available_item = create_disabled_item(&disk_available_label(stats));

    submenu.append(&disk_usage_item);
    submenu.append(&disk_total_item);
    submenu.append(&disk_available_item);
    submenu.append(&create_disabled_item(&format!(
        "Usado: {:.1} GB",
        stats.disk.used_space
    )));

    (
        disk_root_item,
        disk_usage_item,
        disk_total_item,
        disk_available_item,
    )
}

fn append_network_section(
    menu: &gtk::Menu,
    stats: &SystemMetrics,
) -> (gtk::MenuItem, gtk::MenuItem, gtk::MenuItem) {
    let submenu = create_submenu();
    let network_root_item = create_submenu_item(&network_summary_label(stats), &submenu);
    menu.append(&network_root_item);

    submenu.append(&create_disabled_item("🌐 Rede"));
    submenu.append(&create_disabled_item(&format!(
        "Interfaces: {}",
        stats.network.interfaces.len()
    )));

    let total_rx_item = create_disabled_item(&total_rx_label(stats));
    let total_tx_item = create_disabled_item(&total_tx_label(stats));

    submenu.append(&total_rx_item);
    submenu.append(&total_tx_item);

    (network_root_item, total_rx_item, total_tx_item)
}

fn append_system_section(
    menu: &gtk::Menu,
    stats: &SystemMetrics,
) -> (gtk::MenuItem, gtk::MenuItem, gtk::MenuItem) {
    let submenu = create_submenu();
    let uptime_root_item = create_submenu_item(&system_summary_label(stats), &submenu);
    menu.append(&uptime_root_item);

    submenu.append(&create_disabled_item("⏱ Sistema"));
    submenu.append(&create_disabled_item(&format!(
        "Resumo: {}",
        format_uptime(stats.uptime)
    )));

    let uptime_item = create_disabled_item(&uptime_label(stats));
    let load_avg_item = create_disabled_item(&load_average_label(stats));

    submenu.append(&uptime_item);
    submenu.append(&load_avg_item);

    (uptime_root_item, uptime_item, load_avg_item)
}

fn create_system_menu(stats: &SystemMetrics, app: &Application) -> (gtk::Menu, MenuItems) {
    let menu = gtk::Menu::new();
    menu.set_size_request(380, -1);

    let (cpu_root_item, cpu_usage_item) = append_cpu_section(&menu, stats);
    let (mem_root_item, mem_usage_item, swap_item) = append_memory_section(&menu, stats);
    let (disk_root_item, disk_usage_item, disk_total_item, disk_available_item) =
        append_disk_section(&menu, stats);
    let (network_root_item, total_rx_item, total_tx_item) = append_network_section(&menu, stats);
    let (uptime_root_item, uptime_item, load_avg_item) = append_system_section(&menu, stats);

    append_separator(&menu);
    let quit_item = gtk::MenuItem::with_label("Sair");
    menu.append(&quit_item);

    let app_clone = app.clone();
    quit_item.connect_activate(move |_| {
        app_clone.quit();
    });

    menu.show_all();

    let menu_items = MenuItems {
        cpu_root_item: cpu_root_item.clone(),
        cpu_usage_item: cpu_usage_item.clone(),
        mem_root_item: mem_root_item.clone(),
        mem_usage_item: mem_usage_item.clone(),
        disk_root_item: disk_root_item.clone(),
        disk_usage_item: disk_usage_item.clone(),
        disk_total_item: disk_total_item.clone(),
        disk_available_item: disk_available_item.clone(),
        network_root_item: network_root_item.clone(),
        swap_item: swap_item.clone(),
        uptime_root_item: uptime_root_item.clone(),
        uptime_item: uptime_item.clone(),
        load_avg_item: load_avg_item.clone(),
        total_rx_item: total_rx_item.clone(),
        total_tx_item: total_tx_item.clone(),
    };

    (menu, menu_items)
}

fn update_menu_items(menu_items: &MenuItems, stats: &SystemMetrics) {
    menu_items
        .cpu_root_item
        .set_label(&cpu_summary_label(stats));
    menu_items.cpu_usage_item.set_label(&cpu_usage_label(stats));

    menu_items
        .mem_root_item
        .set_label(&memory_summary_label(stats));
    menu_items
        .mem_usage_item
        .set_label(&memory_usage_label(stats));

    if let (Some(swap_item), Some(label)) = (&menu_items.swap_item, swap_label(stats)) {
        swap_item.set_label(&label);
    }

    menu_items
        .disk_root_item
        .set_label(&disk_summary_label(stats));
    menu_items
        .disk_usage_item
        .set_label(&disk_usage_label(stats));
    menu_items
        .disk_total_item
        .set_label(&disk_total_label(stats));
    menu_items
        .disk_available_item
        .set_label(&disk_available_label(stats));

    menu_items
        .network_root_item
        .set_label(&network_summary_label(stats));
    menu_items.total_rx_item.set_label(&total_rx_label(stats));
    menu_items.total_tx_item.set_label(&total_tx_label(stats));

    menu_items
        .uptime_root_item
        .set_label(&system_summary_label(stats));
    menu_items.uptime_item.set_label(&uptime_label(stats));
    menu_items
        .load_avg_item
        .set_label(&load_average_label(stats));
}

fn normalized_usage(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn cpu_icon_color() -> &'static str {
    "#60a5fa"
}

fn memory_icon_color() -> &'static str {
    "#c084fc"
}

fn icon_bar_height(value: f32, max_height: u32) -> u32 {
    let usage = normalized_usage(value);
    if usage <= 0.0 {
        return 0;
    }

    let scaled_height = ((usage / 100.0) * max_height as f32).round() as u32;
    scaled_height.clamp(2, max_height)
}

fn create_status_icon(stats: &SystemMetrics) -> Result<String, Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let icon_width = 52;
    let icon_height = 36;
    let bar_max_width = 42;
    let bar_height = 8;
    let cpu_width = icon_bar_height(stats.cpu.usage_percent, bar_max_width);
    let memory_width = icon_bar_height(stats.memory.usage_percent, bar_max_width);

    let svg_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<svg width=\"{icon_width}\" height=\"{icon_height}\" viewBox=\"0 0 52 36\" xmlns=\"http://www.w3.org/2000/svg\">\
  <rect x=\"1\" y=\"1\" width=\"50\" height=\"34\" rx=\"8\" fill=\"#111827\" stroke=\"#334155\" stroke-width=\"1\"/>\
  <rect x=\"5\" y=\"8\" width=\"42\" height=\"{bar_height}\" rx=\"4\" fill=\"#374151\"/>\
  <rect x=\"5\" y=\"20\" width=\"42\" height=\"{bar_height}\" rx=\"4\" fill=\"#374151\"/>\
  <rect x=\"5\" y=\"8\" width=\"{cpu_width}\" height=\"{bar_height}\" rx=\"4\" fill=\"{}\"/>\
  <rect x=\"5\" y=\"20\" width=\"{memory_width}\" height=\"{bar_height}\" rx=\"4\" fill=\"{}\"/>\
</svg>",
        cpu_icon_color(),
        memory_icon_color()
    );

    let current_file = FILE_TOGGLE.fetch_xor(true, Ordering::Relaxed);
    let file_name = if current_file {
        "monitor_status_icon_a.svg"
    } else {
        "monitor_status_icon_b.svg"
    };
    let temp_path = icon_directory()?.join(file_name);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_path)?;
    file.write_all(svg_content.as_bytes())?;
    file.flush()?;
    Ok(temp_path.to_string_lossy().into_owned())
}

async fn collect_metrics(monitor: &mut SystemMonitor) -> SystemMetrics {
    monitor.update_metrics().await;
    monitor.get_all_metrics()
}

fn collect_initial_metrics() -> Result<SystemMetrics, String> {
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("falha ao criar runtime tokio: {e}"))?;
    let mut monitor = SystemMonitor::new();
    Ok(rt.block_on(collect_metrics(&mut monitor)))
}

fn spawn_metrics_thread(tx: mpsc::Sender<SystemMetrics>) {
    thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(err) => {
                eprintln!("Erro ao criar runtime tokio: {err}");
                return;
            }
        };
        let mut monitor = SystemMonitor::new();

        loop {
            let stats = rt.block_on(collect_metrics(&mut monitor));
            if tx.send(stats).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .application_id("com.monitor.tray")
        .build();

    app.connect_activate(|app| {
        // Create a hidden window (required for some desktop environments)
        let _window = ApplicationWindow::builder()
            .application(app)
            .default_width(1)
            .default_height(1)
            .visible(false)
            .build();

        let stats = match collect_initial_metrics() {
            Ok(stats) => stats,
            Err(err) => {
                eprintln!("Erro ao coletar métricas iniciais: {err}");
                app.quit();
                return;
            }
        };

        // Create compact status icon file
        let icon_path = match create_status_icon(&stats) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Erro ao criar ícone: {e}");
                app.quit();
                return;
            }
        };

        // Create app indicator
        let mut indicator = AppIndicator::new("system-monitor", &icon_path);
        indicator.set_status(AppIndicatorStatus::Active);
        // Remove label since we're using a custom icon
        indicator.set_label("", "");

        // Create initial menu
        let (mut menu, menu_items) = create_system_menu(&stats, app);
        indicator.set_menu(&mut menu);

        // Setup update timer
        let (tx, rx) = mpsc::channel();

        // Stats monitoring thread - com runtime e monitor persistentes
        spawn_metrics_thread(tx);

        // Update UI periodically
        let indicator_rc = Rc::new(RefCell::new(indicator));
        let icon_cache = Rc::new(RefCell::new(IconCache::new()));
        let menu_items_rc = Rc::new(menu_items);

        glib::timeout_add_local(Duration::from_millis(50), move || {
            if let Ok(stats) = rx.try_recv() {
                // Update compact status icon
                if let Ok(icon_path) = icon_cache.borrow_mut().get_icon_path(&stats) {
                    let mut indicator = indicator_rc.borrow_mut();
                    indicator.set_icon_full(&icon_path, "system-monitor");
                    // Force refresh by setting status
                    indicator.set_status(AppIndicatorStatus::Active);
                }

                // Update menu items labels without recreating menu
                update_menu_items(&menu_items_rc, &stats);
            }
            glib::ControlFlow::Continue
        });

        // window.present();
    });
    app.run();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::{
        CpuMetrics, DiskInfo, DiskMetrics, MemoryMetrics, NetworkInterface, NetworkMetrics,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_metrics() -> SystemMetrics {
        let mut interfaces = HashMap::new();
        interfaces.insert(
            "eth0".to_string(),
            NetworkInterface {
                bytes_received: 1024,
                bytes_transmitted: 2048,
                packets_received: 10,
                packets_transmitted: 20,
                errors_received: 0,
                errors_transmitted: 0,
            },
        );

        SystemMetrics {
            cpu: CpuMetrics {
                usage_percent: 42.4,
                core_count: 4,
                per_core_usage: vec![40.0, 41.0, 43.0, 45.0],
                frequency: 3200,
                name: "Test CPU".to_string(),
            },
            memory: MemoryMetrics {
                total_memory: 16.0,
                used_memory: 7.4,
                available_memory: 8.6,
                usage_percent: 46.25,
                total_swap: 4.0,
                used_swap: 1.0,
            },
            disk: DiskMetrics {
                disks: vec![DiskInfo {
                    name: "root".to_string(),
                    mount_point: "/".to_string(),
                    total_space: 100.0,
                    available_space: 40.0,
                    used_space: 60.0,
                    usage_percent: 60.0,
                }],
                total_space: 100.0,
                used_space: 60.0,
                available_space: 40.0,
            },
            network: NetworkMetrics {
                interfaces,
                total_bytes_received: 1024,
                total_bytes_transmitted: 2048,
            },
            uptime: 90061,
            load_average: (0.5, 0.7, 0.9),
        }
    }

    #[test]
    fn test_create_bar_chart_uses_green_indicator_for_low_usage() {
        let bar = create_bar_chart(25.0, 100.0, 10);

        assert!(bar.contains("🟢"));
        assert!(bar.contains("25.0%"));
        assert!(bar.contains("███"));
    }

    #[test]
    fn test_create_bar_chart_uses_yellow_indicator_for_medium_usage() {
        let bar = create_bar_chart(65.0, 100.0, 10);

        assert!(bar.contains("🟡"));
        assert!(bar.contains("65.0%"));
        assert!(bar.contains("██████"));
    }

    #[test]
    fn test_create_bar_chart_clamps_invalid_values() {
        let negative = create_bar_chart(-10.0, 100.0, 10);
        let above_max = create_bar_chart(150.0, 100.0, 10);
        let zero_max = create_bar_chart(50.0, 0.0, 10);
        let nan_value = create_bar_chart(f32::NAN, 100.0, 10);

        assert!(negative.contains("░░░░░░░░░░"));
        assert!(above_max.contains("██████████"));
        assert!(zero_max.contains("░░░░░░░░░░"));
        assert!(nan_value.contains("░░░░░░░░░░"));
    }

    #[test]
    fn test_format_bytes_formats_expected_units() {
        assert_eq!(format_bytes(999.0), "999 B");
        assert_eq!(format_bytes(1024.0), "1.0 KB");
        assert_eq!(format_bytes(1024.0 * 1024.0), "1.0 MB");
        assert_eq!(format_bytes(1024.0 * 1024.0 * 1024.0), "1.0 GB");
    }

    #[test]
    fn test_format_uptime_formats_days_hours_and_minutes() {
        assert_eq!(format_uptime(59), "0m");
        assert_eq!(format_uptime(3661), "1h 1m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn test_icon_palette_returns_expected_colors() {
        assert_eq!(cpu_icon_color(), "#60a5fa");
        assert_eq!(memory_icon_color(), "#c084fc");
    }

    #[test]
    fn test_icon_bar_height_clamps_expected_values() {
        assert_eq!(icon_bar_height(0.0, 16), 0);
        assert_eq!(icon_bar_height(1.0, 16), 2);
        assert_eq!(icon_bar_height(50.0, 16), 8);
        assert_eq!(icon_bar_height(100.0, 16), 16);
        assert_eq!(icon_bar_height(150.0, 16), 16);
    }

    #[test]
    fn test_create_status_icon_writes_compact_svg_with_expected_geometry() {
        let _guard = test_lock().lock().unwrap();
        let stats = sample_metrics();

        let icon_path = create_status_icon(&stats).expect("deve criar ícone SVG");
        let content = fs::read_to_string(&icon_path).expect("deve ler SVG gerado");

        assert!(content.contains("width=\"52\""));
        assert!(content.contains("height=\"36\""));
        assert!(content.contains("stroke=\"#334155\""));
        assert!(content.contains("x=\"5\" y=\"8\" width=\"42\" height=\"8\""));
        assert!(content.contains("x=\"5\" y=\"20\" width=\"42\" height=\"8\""));
        assert!(!content.contains(">C</text>"));
        assert!(!content.contains(">R</text>"));
        assert!(content.contains("#60a5fa"));
        assert!(content.contains("#c084fc"));
    }

    #[test]
    fn test_icon_cache_reuses_path_when_visible_values_do_not_change() {
        let _guard = test_lock().lock().unwrap();
        let stats = sample_metrics();
        let mut cache = IconCache::new();

        let first_path = cache
            .get_icon_path(&stats)
            .expect("deve gerar caminho do primeiro ícone");

        let mut similar_stats = sample_metrics();
        similar_stats.cpu.usage_percent = 42.49;
        similar_stats.memory.usage_percent = 46.49;

        let second_path = cache
            .get_icon_path(&similar_stats)
            .expect("deve reutilizar caminho do ícone em cache");

        assert_eq!(first_path, second_path);
    }

    #[test]
    fn test_icon_cache_refreshes_path_when_memory_usage_changes() {
        let _guard = test_lock().lock().unwrap();
        let stats = sample_metrics();
        let mut cache = IconCache::new();

        let first_path = cache
            .get_icon_path(&stats)
            .expect("deve gerar caminho do primeiro ícone");

        let mut updated_stats = sample_metrics();
        updated_stats.memory.usage_percent = 47.0;

        let second_path = cache
            .get_icon_path(&updated_stats)
            .expect("deve gerar novo caminho quando o uso de memória muda no ícone");

        assert_ne!(first_path, second_path);
    }
}
