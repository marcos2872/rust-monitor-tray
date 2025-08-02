mod monitor;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};

use crate::monitor::{SystemMetrics, SystemMonitor};

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
