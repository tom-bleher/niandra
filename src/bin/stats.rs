//! Music Stats CLI
//!
//! Standalone binary for viewing listening statistics.

use clap::Parser;
use music_analytics::{
    config::Config,
    db::Database,
    display::{
        build_date_range, display_overview, display_top_albums, display_top_artists,
        display_top_tracks, make_bar, print_section,
    },
    error::Result,
};

#[derive(Parser)]
#[command(name = "music-stats")]
#[command(author, version, about = "Display your music listening statistics")]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

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

    /// Show advanced analytics
    #[arg(long)]
    deep: bool,

    /// Show everything
    #[arg(long)]
    full: bool,

    /// Number of items to show in top lists
    #[arg(short, long, default_value = "10")]
    limit: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = if let Some(ref path) = args.config {
        Config::load_from(path)?
    } else {
        Config::load()?
    };

    // Initialize database
    let data_dir = config.data_dir()?;
    let db = Database::new(&config.database, &data_dir).await?;

    // Determine date range
    let (start_date, end_date, period_name) =
        build_date_range(args.all_time, args.week, args.month, args.year);

    // Display header
    println!("\n{}", "*".repeat(50));
    println!("     MUSIC WRAPPED - {}", period_name.to_uppercase());
    println!("{}", "*".repeat(50));

    // Get overview stats
    let overview = db
        .get_overview_stats(start_date.as_deref(), end_date.as_deref())
        .await?;

    if overview.total_plays == 0 {
        println!("\nNo listening data found for {period_name}.");
        println!("Start playing music with music-tracker running!");
        return Ok(());
    }

    // Overview section
    print_section("OVERVIEW");
    display_overview(&overview);

    // Top Artists
    let limit = args.limit;
    print_section(&format!("TOP {limit} ARTISTS"));
    let artists = db
        .get_top_artists(start_date.as_deref(), end_date.as_deref(), limit)
        .await?;
    display_top_artists(&artists, true);

    // Top Albums
    print_section(&format!("TOP {limit} ALBUMS"));
    let albums = db
        .get_top_albums(start_date.as_deref(), end_date.as_deref(), limit)
        .await?;
    display_top_albums(&albums, true);

    // Top Tracks
    print_section(&format!("TOP {limit} TRACKS"));
    let tracks = db
        .get_top_tracks(start_date.as_deref(), end_date.as_deref(), limit)
        .await?;
    display_top_tracks(&tracks, true);

    // Advanced analytics
    if args.deep || args.full {
        print_section("LISTENING PATTERNS");

        // Streaks
        let streaks = db
            .get_listening_streaks(start_date.as_deref(), end_date.as_deref())
            .await?;
        println!("  Current streak:   {} days", streaks.current_streak);
        println!("  Longest streak:   {} days", streaks.longest_streak);
        if let (Some(start), Some(end)) = (&streaks.longest_streak_start, &streaks.longest_streak_end)
        {
            println!("    ({start} to {end})");
        }

        // Night owl
        let night_owl = db
            .get_night_owl_score(start_date.as_deref(), end_date.as_deref())
            .await?;
        println!(
            "  Night owl score:  {:.1}% ({} plays midnight-6am)",
            night_owl.percentage, night_owl.night_plays
        );

        // Skip rate
        let (skipped, total, rate) = db
            .get_skip_rate(start_date.as_deref(), end_date.as_deref())
            .await?;
        if total > 0 {
            println!("  Skip rate:        {rate:.1}% ({skipped}/{total})");
        }

        // Hourly heatmap
        print_section("LISTENING BY TIME");
        let heatmap = db
            .get_hourly_heatmap(start_date.as_deref(), end_date.as_deref())
            .await?;

        let periods = [
            ("Morning (6-12)", 6..12),
            ("Afternoon (12-18)", 12..18),
            ("Evening (18-24)", 18..24),
            ("Night (0-6)", 0..6),
        ];

        let max_period: i64 = periods
            .iter()
            .map(|(_, range)| {
                range
                    .clone()
                    .map(|h| heatmap.hours.get(&h).copied().unwrap_or(0))
                    .sum()
            })
            .max()
            .unwrap_or(1);

        for (name, range) in periods {
            let count: i64 = range
                .map(|h| heatmap.hours.get(&h).copied().unwrap_or(0))
                .sum();
            let bar = make_bar(count, max_period, 25);
            println!("  {name:<18} {bar} {count:>4}");
        }
    }

    println!("\n{}\n", "*".repeat(50));

    Ok(())
}
