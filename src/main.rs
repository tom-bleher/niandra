//! Music Analytics - Main entry point
//!
//! This is the combined CLI that can run as either tracker or stats viewer.

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod analytics;
mod config;
mod context;
mod db;
mod display;
mod error;
mod mpris;
mod track;

use config::Config;
use db::Database;
use error::Result;

#[derive(Parser)]
#[command(name = "music-analytics")]
#[command(author, version, about = "Personal music listening analytics for Linux")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<std::path::PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the music tracker (runs in foreground)
    Track,

    /// Show listening statistics
    Stats {
        /// Show stats for last 7 days
        #[arg(long)]
        week: bool,

        /// Show stats for current month
        #[arg(long)]
        month: bool,

        /// Show stats for a specific year
        #[arg(long)]
        year: Option<i32>,

        /// Show all-time stats
        #[arg(long)]
        all_time: bool,

        /// Number of items to show in top lists
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },

    /// Show or edit configuration
    Config {
        /// Print current configuration
        #[arg(long)]
        show: bool,

        /// Create default configuration file
        #[arg(long)]
        init: bool,
    },

    /// Database operations
    Db {
        /// Show database path and stats
        #[arg(long)]
        info: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Load and validate configuration
    let config = if let Some(ref path) = cli.config {
        Config::load_from(path)?
    } else {
        Config::load()?
    };
    config.validate()?;

    match cli.command {
        Some(Commands::Track) => run_tracker(config).await,

        Some(Commands::Stats {
            week,
            month,
            year,
            all_time,
            limit,
        }) => {
            run_stats(config, week, month, year, all_time, limit).await
        }

        Some(Commands::Config { show, init }) => {
            if init {
                let default_config = Config::default();
                default_config.save()?;
                println!(
                    "Created default configuration at {}",
                    Config::config_path()?.display()
                );
            } else if show {
                let contents = toml::to_string_pretty(&config)?;
                println!("{contents}");
            } else {
                println!("Configuration path: {}", Config::config_path()?.display());
            }
            Ok(())
        }

        Some(Commands::Db { info }) => {
            if info {
                let data_dir = config.data_dir()?;
                let db = Database::new(&config.database, &data_dir).await?;
                let count = db.get_play_count().await?;
                println!("Database path: {}", config.database_path()?.display());
                println!("Total plays: {count}");
            }
            Ok(())
        }

        None => {
            // Default: show stats
            run_stats(config, false, false, None, false, 10).await
        }
    }
}

async fn run_tracker(config: Config) -> Result<()> {
    use tokio::signal;

    let data_dir = config.data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    let db = Database::new(&config.database, &data_dir).await?;

    let monitor = mpris::MprisMonitor::new(
        config.players.clone(),
        config.tracking.clone(),
        db,
    )
    .await?;

    // Handle shutdown signals
    let monitor_handle = std::sync::Arc::new(monitor);
    let monitor_clone = monitor_handle.clone();

    tokio::spawn(async move {
        let _ = signal::ctrl_c().await;
        tracing::info!("Received shutdown signal");
        monitor_clone.stop();
    });

    monitor_handle.run().await
}

async fn run_stats(
    config: Config,
    week: bool,
    month: bool,
    year: Option<i32>,
    all_time: bool,
    limit: u32,
) -> Result<()> {
    let data_dir = config.data_dir()?;
    let db = Database::new(&config.database, &data_dir).await?;

    let (start_date, end_date, period_name) =
        display::build_date_range(all_time, week, month, year);

    // Get and display stats
    let overview = db
        .get_overview_stats(start_date.as_deref(), end_date.as_deref())
        .await?;

    println!("\n{}", "=".repeat(50));
    println!("  MUSIC ANALYTICS - {}", period_name.to_uppercase());
    println!("{}\n", "=".repeat(50));

    if overview.total_plays == 0 {
        println!("No listening data found for this period.");
        println!("Start playing music with the tracker running!");
        return Ok(());
    }

    // Overview
    display::print_section_simple("OVERVIEW");
    display::display_overview(&overview);

    // Top Artists
    display::print_section_simple(&format!("TOP {limit} ARTISTS"));
    let artists = db
        .get_top_artists(start_date.as_deref(), end_date.as_deref(), limit)
        .await?;
    display::display_top_artists(&artists, false);

    // Top Albums
    display::print_section_simple(&format!("TOP {limit} ALBUMS"));
    let albums = db
        .get_top_albums(start_date.as_deref(), end_date.as_deref(), limit)
        .await?;
    display::display_top_albums(&albums, false);

    // Top Tracks
    display::print_section_simple(&format!("TOP {limit} TRACKS"));
    let tracks = db
        .get_top_tracks(start_date.as_deref(), end_date.as_deref(), limit)
        .await?;
    display::display_top_tracks(&tracks, false);

    println!("\n{}\n", "=".repeat(50));

    Ok(())
}
