use demo::{create_app, parse_fault_error_rate, AppState, Metrics};
use std::net::SocketAddr;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() {
    let raw_fault = std::env::var("FAULT_ERROR_RATE").ok();
    let fault_rate = match parse_fault_error_rate(raw_fault.as_deref()) {
        Ok(rate) => rate,
        Err(err) => {
            eprintln!("Configuration error: {err}");
            std::process::exit(1);
        }
    };

    let metrics = match Metrics::new() {
        Ok(m) => m,
        Err(err) => {
            eprintln!("Failed to initialize metrics: {err}");
            std::process::exit(1);
        }
    };

    let state = AppState {
        fault_rate,
        metrics,
    };

    let app = create_app(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(err) => {
            eprintln!("Failed to bind to {addr}: {err}");
            std::process::exit(1);
        }
    };

    let startup_log = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "event": "startup",
        "service": "demo",
        "version": env!("CARGO_PKG_VERSION"),
        "addr": addr.to_string(),
        "fault_error_rate": fault_rate,
    });
    println!("{}", startup_log);

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("Server error: {err}");
        std::process::exit(1);
    }

    let shutdown_log = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "event": "shutdown",
        "service": "demo",
    });
    println!("{}", shutdown_log);
}
