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
    // ram_usage: f32,
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
    // let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
                                                                           // let ram_usage = (used_memory / total_memory * 100.0) as f32;

    SystemStats {
        cpu_usage,
        // ram_usage,
        // ram_total: total_memory,
        ram_used: used_memory,
    }
}

fn create_text_icon(stats: &SystemStats) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;

    // Create SVG with the specified format
    let svg_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<svg width=\"98\" height=\"32\" xmlns=\"http://www.w3.org/2000/svg\">\
        <rect width=\"100%\" height=\"100%\" fill=\"transparent\" />\
          <text x=\"5\" y=\"9\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">C</text>\
          <text x=\"13\" y=\"9\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">P</text>\
          <text x=\"21\" y=\"9\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">U</text>\
          <text x=\"5\" y=\"22\" font-family=\"monospace\" font-size=\"14px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">{:.0}%</text>\
          <text x=\"54\" y=\"9\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">R</text>\
          <text x=\"62\" y=\"9\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">A</text>\
          <text x=\"70\" y=\"9\" font-family=\"monospace\" font-size=\"12px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">M</text>\
          <text x=\"54\" y=\"22\" font-family=\"monospace\" font-size=\"14px\" fill=\"#ffffff\" text-anchor=\"start\" dominant-baseline=\"middle\">{:.1}gb</text>\
        </svg>",
        stats.cpu_usage, stats.ram_used
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
    println!("Iniciando monitor de sistema...");

    let app = Application::builder()
        .application_id("com.monitor.tray")
        .build();

    app.connect_activate(|app| {
        println!("Aplicação ativada");

        // Create a hidden window (required for some desktop environments)
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(1)
            .default_height(1)
            .visible(false)
            .build();

        // Get initial stats
        let stats = get_system_stats();
        // println!("CPU: {:.1}%, RAM: {:.1}%", stats.cpu_usage, stats.ram_usage);

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

        // let stats_item = gtk::MenuItem::with_label(&format!(
        //     "CPU: {:.1}% | RAM: {:.1}GB/{:.1}GB",
        //     stats.cpu_usage, stats.ram_used, stats.ram_total
        // ));
        // stats_item.set_sensitive(false);
        // menu.append(&stats_item);

        let separator = gtk::SeparatorMenuItem::new();
        menu.append(&separator);

        let quit_item = gtk::MenuItem::with_label("Sair");
        menu.append(&quit_item);

        // Connect quit action
        let app_clone = app.clone();
        quit_item.connect_activate(move |_| {
            println!("Saindo...");
            app_clone.quit();
        });

        menu.show_all();
        indicator.set_menu(&mut menu);

        println!("Indicador criado com sucesso!");

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
        // let stats_item_rc = Rc::new(RefCell::new(stats_item));

        glib::timeout_add_local(Duration::from_millis(50), move || {
            if let Ok(stats) = rx.try_recv() {
                // Update text icon
                if let Ok(icon_path) = create_text_icon(&stats) {
                    let mut indicator = indicator_rc.borrow_mut();
                    indicator.set_icon_full(&icon_path, "system-monitor");
                    // Force refresh by setting status
                    indicator.set_status(AppIndicatorStatus::Active);
                }

                // // Update menu item
                // stats_item_rc.borrow().set_label(&format!(
                //     "CPU: {:.1}% | RAM: {:.1}GB/{:.1}GB ({:.1}%)",
                //     stats.cpu_usage, stats.ram_used, stats.ram_total, stats.ram_usage
                // ));

                // println!(
                //     "Atualizado - CPU: {:.1}%, RAM: {:.1}%",
                //     stats.cpu_usage, stats.ram_usage
                // );
            }
            glib::ControlFlow::Continue
        });

        window.present();
    });

    println!("Executando aplicação...");
    app.run();

    Ok(())
}
