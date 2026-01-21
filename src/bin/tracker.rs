//! Music Tracker Daemon
//!
//! Standalone binary for running the MPRIS tracker.

use music_analytics::{config::Config, db::Database, error::Result, mpris::MprisMonitor};
use tokio::signal;
use tracing_subscriber::EnvFilter;

use clap::Parser;

#[derive(Parser)]
#[command(name = "music-tracker")]
#[command(author, version, about = "Music listening tracker daemon")]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Run in foreground (default)
    #[arg(short, long)]
    foreground: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Load configuration
    let config = if let Some(ref path) = args.config {
        Config::load_from(path)?
    } else {
        Config::load()?
    };

    tracing::info!("Music tracker starting...");

    // Initialize database
    let data_dir = config.data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    let db = Database::new(&config.database, &data_dir).await?;
    tracing::info!("Database initialized at {:?}", config.database_path()?);

    // Create MPRIS monitor
    let monitor = MprisMonitor::new(
        config.players.clone(),
        config.tracking.clone(),
        db,
    )
    .await?;

    // Handle shutdown signals
    let monitor = std::sync::Arc::new(monitor);
    let monitor_clone = monitor.clone();

    tokio::spawn(async move {
        let _ = signal::ctrl_c().await;
        tracing::info!("Received shutdown signal, stopping...");
        monitor_clone.stop();
    });

    // Also handle SIGTERM
    #[cfg(unix)]
    {
        let monitor_clone = monitor.clone();
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                tokio::spawn(async move {
                    sigterm.recv().await;
                    tracing::info!("Received SIGTERM, stopping...");
                    monitor_clone.stop();
                });
            }
            Err(e) => {
                tracing::warn!("Failed to register SIGTERM handler: {}. Use Ctrl+C to stop.", e);
            }
        }
    }

    // Run the monitor
    monitor.run().await?;

    tracing::info!("Music tracker stopped");
    Ok(())
}
