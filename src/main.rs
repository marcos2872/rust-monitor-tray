mod monitor;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};
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
        .application_id("com.monitor.desktop")
        .build();

    app.connect_activate(|app| {
        let rt = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(collect_metrics())
        })
        .join()
        .unwrap();

        let stats = rt;

        // Create desktop bar window - even smaller, centered
        let window = ApplicationWindow::builder()
            .application(app)
            .title("System Monitor")
            .default_width(250) // Even smaller width
            .default_height(32) // Thinner
            .decorated(false)
            .resizable(false)
            .build();

        // Position window at top left
        window.move_(10, 5); // Top left corner with small margin

        // Keep window on top and make it truly transparent
        window.set_keep_above(true);
        window.stick();
        window.set_type_hint(gtk::gdk::WindowTypeHint::Dock);

        // Enable transparency with compositor support
        if let Some(screen) = gtk::prelude::WidgetExt::screen(&window) {
            if let Some(visual) = screen.rgba_visual() {
                window.set_visual(Some(&visual));
                window.set_app_paintable(true);
            } else {
                // Fallback for systems without compositor
                window.set_app_paintable(false);
            }
        }

        // Create horizontal layout with very tight spacing
        let hbox = GtkBox::new(Orientation::Horizontal, 12);
        hbox.set_halign(gtk::Align::Center);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);
        hbox.set_margin_top(5);
        hbox.set_margin_bottom(5);

        // CPU label with fixed width
        let cpu_label = Label::new(Some("CPU 100%"));
        cpu_label.set_width_chars(8);
        cpu_label.set_justify(gtk::Justification::Center);
        cpu_label.set_markup(&format!(
            "<span color='#ffffff' size='9000' weight='bold'>CPU {:.0}%</span>",
            stats.cpu.usage_percent
        ));
        hbox.pack_start(&cpu_label, false, false, 0);

        // Memory label with fixed width
        let mem_label = Label::new(Some("RAM 99.9G"));
        mem_label.set_width_chars(9);
        mem_label.set_justify(gtk::Justification::Center);
        mem_label.set_markup(&format!(
            "<span color='#ffffff' size='9000' weight='bold'>RAM {:.1}G</span>",
            stats.memory.used_memory
        ));
        hbox.pack_start(&mem_label, false, false, 0);

        // Network RX label with fixed width
        let net_rx_label = Label::new(Some("↓999GB"));
        net_rx_label.set_width_chars(7);
        net_rx_label.set_justify(gtk::Justification::Center);
        net_rx_label.set_markup(&format!(
            "<span color='#87ceeb' size='8000' weight='bold'>↓{}</span>",
            format_bytes(stats.network.total_bytes_received as f64)
        ));
        hbox.pack_start(&net_rx_label, false, false, 0);

        // Network TX label with fixed width
        let net_tx_label = Label::new(Some("↑999GB"));
        net_tx_label.set_width_chars(7);
        net_tx_label.set_justify(gtk::Justification::Center);
        net_tx_label.set_markup(&format!(
            "<span color='#87ceeb' size='8000' weight='bold'>↑{}</span>",
            format_bytes(stats.network.total_bytes_transmitted as f64)
        ));
        hbox.pack_start(&net_tx_label, false, false, 0);

        // Uptime label with fixed width
        let uptime_label = Label::new(Some("UP 999d 23h"));
        uptime_label.set_width_chars(11);
        uptime_label.set_justify(gtk::Justification::Center);
        uptime_label.set_markup(&format!(
            "<span color='#ffd700' size='8000'>UP {}</span>",
            format_uptime(stats.uptime)
        ));
        hbox.pack_end(&uptime_label, false, false, 0);

        // Set semi-transparent black background
        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(
                b"
            window {
                background: rgba(0, 0, 0, 0.8);
                border-radius: 8px;
                border: 2px solid rgba(255, 255, 255, 0.9);
                box-shadow: 0 3px 10px rgba(0, 0, 0, 0.7);
            }
            box {
                background: rgba(0, 0, 0, 0.3);
                border-radius: 6px;
                padding: 2px;
            }
            label {
                min-width: 60px;
                background: rgba(0, 0, 0, 0.2);
                border-radius: 4px;
                padding: 2px;
            }
        ",
            )
            .unwrap();

        let style_context = window.style_context();
        style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        window.add(&hbox);

        // Setup update timer
        let (tx, rx) = mpsc::channel();

        // Stats monitoring thread
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            loop {
                let stats = rt.block_on(collect_metrics());

                if tx.send(stats).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
            }
        });

        // Clone labels for the closure
        let cpu_label_clone = cpu_label.clone();
        let mem_label_clone = mem_label.clone();
        let net_rx_label_clone = net_rx_label.clone();
        let net_tx_label_clone = net_tx_label.clone();
        let uptime_label_clone = uptime_label.clone();

        // Update UI periodically
        glib::timeout_add_local(Duration::from_millis(100), move || {
            if let Ok(stats) = rx.try_recv() {
                // Determine colors based on usage with better color scheme
                let cpu_color = if stats.cpu.usage_percent < 50.0 {
                    "#7fffd4"
                }
                // Aquamarine
                else if stats.cpu.usage_percent < 80.0 {
                    "#ffd700"
                }
                // Gold
                else {
                    "#ff6b6b"
                }; // Coral red

                let mem_color = if stats.memory.usage_percent < 50.0 {
                    "#7fffd4"
                }
                // Aquamarine
                else if stats.memory.usage_percent < 80.0 {
                    "#ffd700"
                }
                // Gold
                else {
                    "#ff6b6b"
                }; // Coral red

                // Update labels with very compact styling
                cpu_label_clone.set_markup(&format!(
                    "<span color='{}' size='9000' weight='bold'>CPU {:.0}%</span>",
                    cpu_color, stats.cpu.usage_percent
                ));
                mem_label_clone.set_markup(&format!(
                    "<span color='{}' size='9000' weight='bold'>RAM {:.1}G</span>",
                    mem_color, stats.memory.used_memory
                ));
                net_rx_label_clone.set_markup(&format!(
                    "<span color='#87ceeb' size='8000' weight='bold'>↓{}</span>",
                    format_bytes(stats.network.total_bytes_received as f64)
                ));
                net_tx_label_clone.set_markup(&format!(
                    "<span color='#87ceeb' size='8000' weight='bold'>↑{}</span>",
                    format_bytes(stats.network.total_bytes_transmitted as f64)
                ));
                uptime_label_clone.set_markup(&format!(
                    "<span color='#ffd700' size='8000'>UP {}</span>",
                    format_uptime(stats.uptime)
                ));
            }
            glib::ControlFlow::Continue
        });

        window.show_all();
    });
    app.run();

    Ok(())
}
