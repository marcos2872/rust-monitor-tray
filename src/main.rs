use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use sysinfo::System;

struct SystemStats {
    cpu_usage: f32,
    ram_usage: f32,
    // ram_total: f64,
    ram_used: f64,
}

fn get_system_stats() -> SystemStats {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Wait a bit and refresh again to get accurate CPU readings
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_all();

    let cpu_usage = sys.global_cpu_usage();
    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
    let ram_usage = (used_memory / total_memory * 100.0) as f32;

    SystemStats {
        cpu_usage,
        ram_usage,
        // ram_total: total_memory,
        ram_used: used_memory,
    }
}

fn create_text_icon(stats: &SystemStats) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;

    // Determine CPU color based on usage
    let cpu_color = if stats.cpu_usage < 50.0 {
        "#ffffff" // White for low usage
    } else if stats.cpu_usage < 80.0 {
        "#ffff00" // Yellow for medium usage
    } else {
        "#ff0000" // Red for high usage
    };

    // Determine RAM color based on actual usage percentage
    let ram_color = if stats.ram_usage < 50.0 {
        "#ffffff" // White for low usage
    } else if stats.ram_usage < 80.0 {
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
        cpu_color, stats.cpu_usage, ram_color, stats.ram_used
    );

    // Use timestamp to ensure unique file for each update
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_path = format!("/tmp/monitor_text_icon_{}.svg", timestamp);
    let mut file = std::fs::File::create(&temp_path)?;
    file.write_all(svg_content.as_bytes())?;

    Ok(temp_path)
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

        // Get initial stats
        let stats = get_system_stats();

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

        // Create menu
        let mut menu = gtk::Menu::new();

        let separator = gtk::SeparatorMenuItem::new();
        menu.append(&separator);

        let quit_item = gtk::MenuItem::with_label("Sair");
        menu.append(&quit_item);

        // Connect quit action
        let app_clone = app.clone();
        quit_item.connect_activate(move |_| {
            app_clone.quit();
        });

        menu.show_all();
        indicator.set_menu(&mut menu);

        // Setup update timer
        let (tx, rx) = mpsc::channel();

        // Stats monitoring thread
        thread::spawn(move || loop {
            let stats = get_system_stats();
            if tx.send(stats).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        });

        // Update UI periodically
        let indicator_rc = Rc::new(RefCell::new(indicator));

        glib::timeout_add_local(Duration::from_millis(50), move || {
            if let Ok(stats) = rx.try_recv() {
                // Update text icon
                if let Ok(icon_path) = create_text_icon(&stats) {
                    let mut indicator = indicator_rc.borrow_mut();
                    indicator.set_icon_full(&icon_path, "system-monitor");
                    // Force refresh by setting status
                    indicator.set_status(AppIndicatorStatus::Active);
                }
            }
            glib::ControlFlow::Continue
        });

        // window.present();
    });
    app.run();

    Ok(())
}
