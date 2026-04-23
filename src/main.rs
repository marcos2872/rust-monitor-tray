use std::error::Error;

use monitor_tray::{collect_metrics_once_json, dbus::run_dbus_service};

fn print_help() {
    println!(
        "monitor-tray\n\nUso:\n  monitor-tray --dbus    Inicia o backend DBus para o Plasmoid KDE\n  monitor-tray --json    Imprime uma amostra de métricas em JSON\n  monitor-tray --help    Exibe esta ajuda\n\nSem argumentos, o binário inicia em modo DBus."
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next();

    if let Some(extra) = args.next() {
        return Err(format!("argumento inesperado: {extra}").into());
    }

    match mode.as_deref() {
        None | Some("--dbus") => run_dbus_service().await,
        Some("--json") => {
            println!("{}", collect_metrics_once_json().await?);
            Ok(())
        }
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some(other) => {
            Err(format!("modo inválido: {other}. Use --help para ver as opções.").into())
        }
    }
}
