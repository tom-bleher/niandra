//! GTK4/Libadwaita GUI binary for Music Analytics
//!
//! Run with: `cargo run --bin music-analytics-gui --features gui`

fn main() -> gtk4::glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    music_analytics::gui::run()
}
