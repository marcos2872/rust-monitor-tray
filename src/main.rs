mod monitor;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
// use tokio::time;

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
static FILE_TOGGLE: AtomicBool = AtomicBool::new(false);

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
        println!("Ge {:?}", stats.disk.disks);

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

        // === INFORMAÇÕES DA CPU ===
        let cpu_title = gtk::MenuItem::with_label("=== PROCESSADOR ===");
        cpu_title.set_sensitive(false);
        menu.append(&cpu_title);

        let cpu_model_item = gtk::MenuItem::with_label(&format!("Modelo: {}", stats.cpu.name));
        cpu_model_item.set_sensitive(false);
        menu.append(&cpu_model_item);

        let cpu_cores_item = gtk::MenuItem::with_label(&format!("Cores: {}", stats.cpu.core_count));
        cpu_cores_item.set_sensitive(false);
        menu.append(&cpu_cores_item);

        let cpu_freq_item =
            gtk::MenuItem::with_label(&format!("Freq. Máx: {}", stats.cpu.frequency));
        cpu_freq_item.set_sensitive(false);
        menu.append(&cpu_freq_item);

        let separator2 = gtk::SeparatorMenuItem::new();
        menu.append(&separator2);

        let separator3 = gtk::SeparatorMenuItem::new();
        menu.append(&separator3);

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

        glib::timeout_add_local(Duration::from_millis(50), move || {
            if let Ok(stats) = rx.try_recv() {
                // Update text icon
                if let Ok(icon_path) = icon_cache.borrow_mut().get_icon_path(&stats) {
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
