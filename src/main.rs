mod monitor;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::monitor::{SystemMetrics, SystemMonitor};

struct IconCache {
    last_cpu: Option<u32>,
    last_ram: Option<u32>,
    current_path: Option<String>,
}

struct MenuItems {
    cpu_usage_item: gtk::MenuItem,
    mem_usage_item: gtk::MenuItem,
    swap_item: Option<gtk::MenuItem>,
    uptime_item: gtk::MenuItem,
    total_rx_item: gtk::MenuItem,
    total_tx_item: gtk::MenuItem,
}
static FILE_TOGGLE: AtomicBool = AtomicBool::new(false);

impl IconCache {
    fn new() -> Self {
        IconCache {
            last_cpu: None,
            last_ram: None,
            current_path: None,
        }
    }

    fn get_icon_path(
        &mut self,
        stats: &SystemMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cpu_rounded = stats.cpu.usage_percent.round() as u32;
        let ram_rounded = stats.memory.used_memory as u32; // 1 decimal place

        // Check if we need to update
        let needs_update = self.last_cpu != Some(cpu_rounded)
            || self.last_ram != Some(ram_rounded)
            || self.current_path.is_none();

        if needs_update {
            // Generate new icon
            let path = create_text_icon(stats)?;
            self.last_cpu = Some(cpu_rounded);
            self.last_ram = Some(ram_rounded);
            self.current_path = Some(path);
        }

        Ok(self.current_path.as_ref().unwrap().clone())
    }
}
fn create_bar_chart(value: f32, max_value: f32, width: u32) -> String {
    let filled_width = ((value / max_value) * width as f32) as u32;
    let empty_width = width - filled_width;

    let filled_char = if value < 50.0 {
        "🟢" // Green circle for 0-50%
    } else if value < 80.0 {
        "🟡" // Yellow circle for 50-80%
    } else {
        "🔴" // Red circle for 80%+
    };

    // Create the bar using | characters
    let filled_bar = "|".repeat(filled_width as usize);
    let empty_bar = "-".repeat(empty_width as usize);

    format!("{} [{}{}]", filled_char, filled_bar, empty_bar)
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

fn create_system_menu(stats: &SystemMetrics, app: &Application) -> (gtk::Menu, MenuItems) {
    let menu = gtk::Menu::new();

    // === INFORMAÇÕES DA CPU ===
    let cpu_title = gtk::MenuItem::with_label("=== PROCESSADOR ===");
    cpu_title.set_sensitive(false);
    menu.append(&cpu_title);

    let cpu_model_item = gtk::MenuItem::with_label(&format!("Modelo: {}", stats.cpu.name));
    cpu_model_item.set_sensitive(false);
    menu.append(&cpu_model_item);

    let cpu_usage_bar = create_bar_chart(stats.cpu.usage_percent, 100.0, 40);
    let cpu_usage_item = gtk::MenuItem::with_label(&format!("{}", cpu_usage_bar));
    cpu_usage_item.set_sensitive(false);
    menu.append(&cpu_usage_item);

    let separator_cpu = gtk::SeparatorMenuItem::new();
    menu.append(&separator_cpu);

    // === INFORMAÇÕES DA MEMÓRIA ===
    let memory_title = gtk::MenuItem::with_label("=== MEMÓRIA ===");
    memory_title.set_sensitive(false);
    menu.append(&memory_title);

    let mem_total_item =
        gtk::MenuItem::with_label(&format!("Total: {:.1} GB", stats.memory.total_memory));
    mem_total_item.set_sensitive(false);
    menu.append(&mem_total_item);

    let mem_usage_bar = create_bar_chart(stats.memory.usage_percent, 100.0, 40);
    let mem_usage_item = gtk::MenuItem::with_label(&format!("{}", mem_usage_bar));
    mem_usage_item.set_sensitive(false);
    menu.append(&mem_usage_item);

    // SWAP
    let swap_item = if stats.memory.total_swap > 0.0 {
        let swap_usage_percent = if stats.memory.total_swap > 0.0 {
            (stats.memory.used_swap / stats.memory.total_swap) * 100.0
        } else {
            0.0
        };
        let item = gtk::MenuItem::with_label(&format!(
            "SWAP: {:.1}/{:.1} GB ({:.1}%)",
            stats.memory.used_swap, stats.memory.total_swap, swap_usage_percent
        ));
        item.set_sensitive(false);
        menu.append(&item);
        Some(item)
    } else {
        None
    };

    let separator_mem = gtk::SeparatorMenuItem::new();
    menu.append(&separator_mem);

    // === INFORMAÇÕES DOS DISCOS ===
    let disk_title = gtk::MenuItem::with_label("=== ARMAZENAMENTO ===");
    disk_title.set_sensitive(false);
    menu.append(&disk_title);

    let disk_total_item =
        gtk::MenuItem::with_label(&format!("Total: {:.1} GB", stats.disk.total_space));
    disk_total_item.set_sensitive(false);
    menu.append(&disk_total_item);

    let disk_available_item =
        gtk::MenuItem::with_label(&format!("Disponível: {:.1} GB", stats.disk.available_space));
    disk_available_item.set_sensitive(false);
    menu.append(&disk_available_item);

    let separator_disk = gtk::SeparatorMenuItem::new();
    menu.append(&separator_disk);

    // === INFORMAÇÕES DE REDE ===
    let network_title = gtk::MenuItem::with_label("=== REDE ===");
    network_title.set_sensitive(false);
    menu.append(&network_title);

    let total_rx_item = gtk::MenuItem::with_label(&format!(
        "Total RX: ↓{}",
        format_bytes(stats.network.total_bytes_received as f64)
    ));
    total_rx_item.set_sensitive(false);
    menu.append(&total_rx_item);

    let total_tx_item = gtk::MenuItem::with_label(&format!(
        "Total TX: ↑{}",
        format_bytes(stats.network.total_bytes_transmitted as f64)
    ));
    total_tx_item.set_sensitive(false);
    menu.append(&total_tx_item);

    let separator_net = gtk::SeparatorMenuItem::new();
    menu.append(&separator_net);

    // === INFORMAÇÕES DO SISTEMA ===
    let system_title = gtk::MenuItem::with_label("=== SISTEMA ===");
    system_title.set_sensitive(false);
    menu.append(&system_title);

    let uptime_item =
        gtk::MenuItem::with_label(&format!("Uptime: {}", format_uptime(stats.uptime)));
    uptime_item.set_sensitive(false);
    menu.append(&uptime_item);

    let separator_final = gtk::SeparatorMenuItem::new();
    menu.append(&separator_final);

    let quit_item = gtk::MenuItem::with_label("Sair");
    menu.append(&quit_item);

    // Connect quit action
    let app_clone = app.clone();
    quit_item.connect_activate(move |_| {
        app_clone.quit();
    });

    menu.show_all();

    let menu_items = MenuItems {
        cpu_usage_item: cpu_usage_item.clone(),
        mem_usage_item: mem_usage_item.clone(),
        swap_item: swap_item.clone(),
        uptime_item: uptime_item.clone(),
        total_rx_item: total_rx_item.clone(),
        total_tx_item: total_tx_item.clone(),
    };

    (menu, menu_items)
}

fn update_menu_items(menu_items: &MenuItems, stats: &SystemMetrics) {
    // Update CPU usage
    let cpu_usage_bar = create_bar_chart(stats.cpu.usage_percent, 100.0, 40);
    menu_items
        .cpu_usage_item
        .set_label(&format!("{}", cpu_usage_bar));

    // Update memory usage
    let mem_usage_bar = create_bar_chart(stats.memory.usage_percent, 100.0, 40);
    menu_items
        .mem_usage_item
        .set_label(&format!("{}", mem_usage_bar));

    // Update SWAP if exists
    if let Some(swap_item) = &menu_items.swap_item {
        if stats.memory.total_swap > 0.0 {
            let swap_usage_percent = (stats.memory.used_swap / stats.memory.total_swap) * 100.0;
            swap_item.set_label(&format!(
                "SWAP: {:.1}/{:.1} GB ({:.1}%)",
                stats.memory.used_swap, stats.memory.total_swap, swap_usage_percent
            ));
        }
    }

    // Update uptime
    menu_items
        .uptime_item
        .set_label(&format!("Uptime: {}", format_uptime(stats.uptime)));

    // Update network
    menu_items.total_rx_item.set_label(&format!(
        "Total RX: ↓{}",
        format_bytes(stats.network.total_bytes_received as f64)
    ));
    menu_items.total_tx_item.set_label(&format!(
        "Total TX: ↑{}",
        format_bytes(stats.network.total_bytes_transmitted as f64)
    ));
}

fn create_text_icon(stats: &SystemMetrics) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;

    // Determine CPU color based on usage
    let cpu_color = if stats.cpu.usage_percent < 50.0 {
        "#ffffff" // White for low usage
    } else if stats.cpu.usage_percent < 80.0 {
        "#ffff00" // Yellow for medium usage
    } else {
        "#ff0000" // Red for high usage
    };

    // Determine RAM color based on actual usage percentage
    let ram_color = if stats.memory.usage_percent < 50.0 {
        "#ffffff" // White for low usage
    } else if stats.memory.usage_percent < 80.0 {
        "#ffff00" // Yellow for medium usage
    } else {
        "#ff0000" // Red for high usage
    };

    // Create SVG with the specified format
    let svg_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<svg width=\"120\" height=\"32\" xmlns=\"http://www.w3.org/2000/svg\">\
        <rect width=\"100%\" height=\"100%\" fill=\"transparent\" />\
          <text x=\"5\" y=\"13\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" font-weight=\"bold\" text-anchor=\"start\" dominant-baseline=\"middle\">CPU</text>\
          <text x=\"5\" y=\"27\" font-family=\"monospace\" font-size=\"14px\" fill=\"{}\" font-weight=\"bold\" text-anchor=\"start\" dominant-baseline=\"middle\">{:.0}%</text>\
          <text x=\"54\" y=\"13\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" font-weight=\"bold\" text-anchor=\"start\" dominant-baseline=\"middle\">RAM</text>\
          <text x=\"54\" y=\"27\" font-family=\"monospace\" font-size=\"14px\" fill=\"{}\" font-weight=\"bold\" text-anchor=\"start\" dominant-baseline=\"middle\">{:.1}gb</text>\
        </svg>",
        cpu_color, stats.cpu.usage_percent, ram_color, stats.memory.used_memory
    );

    let current_file = FILE_TOGGLE.fetch_xor(true, Ordering::Relaxed);
    let temp_path = if current_file {
        "/tmp/monitor_text_icon_a.svg"
    } else {
        "/tmp/monitor_text_icon_b.svg"
    };

    let mut file = std::fs::File::create(temp_path)?;
    file.write_all(svg_content.as_bytes())?;
    file.flush()?;
    Ok(temp_path.to_string())
}

async fn collect_metrics() -> SystemMetrics {
    let mut monitor = SystemMonitor::new();
    monitor.update_metrics().await;
    monitor.get_all_metrics()
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

        let rt = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(collect_metrics())
        })
        .join()
        .unwrap();

        let stats = rt;

        // Create text icon file
        let icon_path = match create_text_icon(&stats) {
            Ok(path) => path,
            Err(e) => {
                println!("Erro ao criar ícone: {}", e);
                return;
            }
        };

        // Create app indicator
        let mut indicator = AppIndicator::new("system-monitor", &icon_path);
        indicator.set_status(AppIndicatorStatus::Active);
        // Remove label since we're showing text in the icon itself
        indicator.set_label("", "");

        // Create initial menu
        let (mut menu, menu_items) = create_system_menu(&stats, app);
        indicator.set_menu(&mut menu);

        // Setup update timer
        let (tx, rx) = mpsc::channel();

        // Stats monitoring thread - com runtime próprio
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            loop {
                let stats = rt.block_on(collect_metrics());

                if tx.send(stats).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(500));
            }
        });

        // Update UI periodically
        let indicator_rc = Rc::new(RefCell::new(indicator));
        let icon_cache = Rc::new(RefCell::new(IconCache::new()));
        let menu_items_rc = Rc::new(menu_items);

        glib::timeout_add_local(Duration::from_millis(50), move || {
            if let Ok(stats) = rx.try_recv() {
                // Update text icon
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
