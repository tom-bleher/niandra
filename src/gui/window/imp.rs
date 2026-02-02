//! GObject implementation for MusicAnalyticsWindow

use std::cell::{Cell, RefCell};

use async_channel::{Receiver, Sender};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

use crate::db::{AlbumStats, ArtistStats, OverviewStats, TrackStats};
use crate::gui::views::{HeatmapView, InsightsView, OverviewView, TopListsView};
use crate::gui::widgets::ContributionData;
use crate::gui::window::{DateFilter, HeatmapData, InsightsData};
use crate::{Config, Database};

/// Messages sent from the async data loading thread
#[derive(Debug)]
pub enum DataMessage {
    Overview(OverviewStats),
    Artists(Vec<ArtistStats>),
    Albums(Vec<AlbumStats>),
    Tracks(Vec<TrackStats>),
    Insights(InsightsData),
    Heatmap(HeatmapData),
    Contribution(ContributionData),
    Error(String),
}

pub struct MusicAnalyticsWindow {
    // UI components
    pub(super) view_stack: adw::ViewStack,
    pub(super) date_dropdown: gtk4::DropDown,
    pub(super) toast_overlay: adw::ToastOverlay,

    // Views
    pub(super) overview_view: RefCell<Option<OverviewView>>,
    pub(super) artists_view: RefCell<Option<TopListsView>>,
    pub(super) albums_view: RefCell<Option<TopListsView>>,
    pub(super) tracks_view: RefCell<Option<TopListsView>>,
    pub(super) insights_view: RefCell<Option<InsightsView>>,
    pub(super) heatmap_view: RefCell<Option<HeatmapView>>,

    // Data channel
    pub(super) sender: Sender<DataMessage>,
    pub(super) receiver: Receiver<DataMessage>,

    // State
    pub(super) date_filter: Cell<DateFilter>,
}

impl Default for MusicAnalyticsWindow {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();

        Self {
            view_stack: adw::ViewStack::new(),
            date_dropdown: gtk4::DropDown::from_strings(&[
                "Today",
                "Past Week",
                "Past Month",
                "Past Year",
                "All Time",
            ]),
            toast_overlay: adw::ToastOverlay::new(),

            overview_view: RefCell::new(None),
            artists_view: RefCell::new(None),
            albums_view: RefCell::new(None),
            tracks_view: RefCell::new(None),
            insights_view: RefCell::new(None),
            heatmap_view: RefCell::new(None),

            sender,
            receiver,

            date_filter: Cell::new(DateFilter::AllTime),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for MusicAnalyticsWindow {
    const NAME: &'static str = "MusicAnalyticsWindow";
    type Type = super::MusicAnalyticsWindow;
    type ParentType = adw::ApplicationWindow;
}

impl ObjectImpl for MusicAnalyticsWindow {
    fn constructed(&self) {
        self.parent_constructed();
        self.setup_ui();
        self.setup_data_channel();
        self.init_database();
    }
}

impl WidgetImpl for MusicAnalyticsWindow {}

impl WindowImpl for MusicAnalyticsWindow {}

impl ApplicationWindowImpl for MusicAnalyticsWindow {}

impl AdwApplicationWindowImpl for MusicAnalyticsWindow {}

impl MusicAnalyticsWindow {
    fn setup_ui(&self) {
        let window = self.obj();

        // Configure window
        window.set_default_size(900, 700);
        window.set_size_request(360, 400);  // Minimum size
        window.set_title(Some("Niandra"));

        // Set default date filter to All Time (index 4)
        self.date_dropdown.set_selected(4);

        // Create the main toolbar view
        let toolbar_view = adw::ToolbarView::new();

        // Create header bar with view switcher
        let header_bar = adw::HeaderBar::new();

        // View switcher for navigation (in header for wide layouts)
        let view_switcher = adw::ViewSwitcher::new();
        view_switcher.set_stack(Some(&self.view_stack));
        view_switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
        header_bar.set_title_widget(Some(&view_switcher));

        // Date filter dropdown
        self.date_dropdown.set_tooltip_text(Some("Time period filter"));
        header_bar.pack_end(&self.date_dropdown);

        // Menu button
        let menu_button = gtk4::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_menu_model(Some(&self.create_app_menu()));
        header_bar.pack_end(&menu_button);

        toolbar_view.add_top_bar(&header_bar);

        // Create views and add to stack
        self.setup_views();

        // View switcher bar for narrow layouts
        let view_switcher_bar = adw::ViewSwitcherBar::new();
        view_switcher_bar.set_stack(Some(&self.view_stack));
        view_switcher_bar.set_reveal(true);

        // Set content
        self.toast_overlay.set_child(Some(&self.view_stack));
        toolbar_view.set_content(Some(&self.toast_overlay));
        toolbar_view.add_bottom_bar(&view_switcher_bar);

        window.set_content(Some(&toolbar_view));

        // Connect date filter change
        self.date_dropdown.connect_selected_notify(
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |dropdown| {
                    let filter = match dropdown.selected() {
                        0 => DateFilter::Today,
                        1 => DateFilter::Week,
                        2 => DateFilter::Month,
                        3 => DateFilter::Year,
                        _ => DateFilter::AllTime,
                    };
                    imp.date_filter.set(filter);
                    imp.reload_data();
                }
            ),
        );

        // Add breakpoint for responsive layout
        let breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
            adw::BreakpointConditionLengthType::MaxWidth,
            550.0,
            adw::LengthUnit::Sp,
        ));

        // Apply styles to views when narrow
        breakpoint.add_setter(&self.view_stack, "margin-start", Some(&0i32.to_value()));
        breakpoint.add_setter(&self.view_stack, "margin-end", Some(&0i32.to_value()));

        window.add_breakpoint(breakpoint);
    }

    fn setup_views(&self) {
        // Overview view
        let overview = OverviewView::new();
        self.view_stack
            .add_titled(&overview, Some("overview"), "Overview")
            .set_icon_name(Some("view-grid-symbolic"));
        *self.overview_view.borrow_mut() = Some(overview);

        // Artists view
        let artists = TopListsView::new_artists();
        self.view_stack
            .add_titled(&artists, Some("artists"), "Artists")
            .set_icon_name(Some("avatar-default-symbolic"));
        *self.artists_view.borrow_mut() = Some(artists);

        // Albums view
        let albums = TopListsView::new_albums();
        self.view_stack
            .add_titled(&albums, Some("albums"), "Albums")
            .set_icon_name(Some("media-optical-symbolic"));
        *self.albums_view.borrow_mut() = Some(albums);

        // Tracks view
        let tracks = TopListsView::new_tracks();
        self.view_stack
            .add_titled(&tracks, Some("tracks"), "Tracks")
            .set_icon_name(Some("emblem-music-symbolic"));
        *self.tracks_view.borrow_mut() = Some(tracks);

        // Insights view
        let insights = InsightsView::new();
        self.view_stack
            .add_titled(&insights, Some("insights"), "Insights")
            .set_icon_name(Some("view-reveal-symbolic"));
        *self.insights_view.borrow_mut() = Some(insights);

        // Heatmap view
        let heatmap = HeatmapView::new();
        self.view_stack
            .add_titled(&heatmap, Some("heatmap"), "Heatmap")
            .set_icon_name(Some("weather-clear-symbolic"));
        *self.heatmap_view.borrow_mut() = Some(heatmap);
    }

    fn create_app_menu(&self) -> gtk4::gio::Menu {
        let menu = gtk4::gio::Menu::new();
        menu.append(Some("About"), Some("app.about"));
        menu.append(Some("Quit"), Some("app.quit"));
        menu
    }

    fn setup_data_channel(&self) {
        let receiver = self.receiver.clone();

        // Process incoming data messages on the main thread
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            async move {
                while let Ok(message) = receiver.recv().await {
                    imp.handle_data_message(message);
                }
            }
        ));
    }

    fn handle_data_message(&self, message: DataMessage) {
        match message {
            DataMessage::Overview(stats) => {
                if let Some(view) = self.overview_view.borrow().as_ref() {
                    view.set_stats(&stats);
                }
            }
            DataMessage::Artists(artists) => {
                if let Some(view) = self.artists_view.borrow().as_ref() {
                    view.set_artist_data(&artists);
                }
            }
            DataMessage::Albums(albums) => {
                if let Some(view) = self.albums_view.borrow().as_ref() {
                    view.set_album_data(&albums);
                }
            }
            DataMessage::Tracks(tracks) => {
                if let Some(view) = self.tracks_view.borrow().as_ref() {
                    view.set_track_data(&tracks);
                }
            }
            DataMessage::Insights(data) => {
                if let Some(view) = self.insights_view.borrow().as_ref() {
                    view.set_data(&data);
                }
            }
            DataMessage::Heatmap(data) => {
                if let Some(view) = self.heatmap_view.borrow().as_ref() {
                    view.set_data(&data);
                }
            }
            DataMessage::Contribution(data) => {
                if let Some(view) = self.heatmap_view.borrow().as_ref() {
                    view.set_contribution_data(data);
                }
            }
            DataMessage::Error(err) => {
                self.show_error(&err);
            }
        }
    }

    fn init_database(&self) {
        // Load data immediately - reload_data handles database connection and errors
        self.reload_data();
    }

    pub fn reload_data(&self) {
        let sender = self.sender.clone();
        let filter = self.date_filter.get();
        let range = filter.to_date_range();
        let (start_date, end_date) = range.to_sql_tuple_with_end_time();

        // Show loading state on views
        if let Some(view) = self.overview_view.borrow().as_ref() {
            view.set_loading(true);
        }
        if let Some(view) = self.artists_view.borrow().as_ref() {
            view.set_loading(true);
        }
        if let Some(view) = self.albums_view.borrow().as_ref() {
            view.set_loading(true);
        }
        if let Some(view) = self.tracks_view.borrow().as_ref() {
            view.set_loading(true);
        }
        if let Some(view) = self.insights_view.borrow().as_ref() {
            view.set_loading(true);
        }
        if let Some(view) = self.heatmap_view.borrow().as_ref() {
            view.set_loading(true);
        }

        // Load data in tokio runtime
        crate::gui::runtime().spawn(async move {
            let config = match Config::load() {
                Ok(c) => c,
                Err(e) => {
                    let _ = sender.send(DataMessage::Error(format!("Config error: {e}"))).await;
                    return;
                }
            };

            let data_dir = match config.data_dir() {
                Ok(d) => d,
                Err(e) => {
                    let _ = sender.send(DataMessage::Error(format!("Data dir error: {e}"))).await;
                    return;
                }
            };

            let db = match Database::new(&config.database, &data_dir).await {
                Ok(db) => db,
                Err(e) => {
                    let _ = sender.send(DataMessage::Error(format!("Database error: {e}"))).await;
                    return;
                }
            };

            // Load overview stats
            if let Ok(overview) = db
                .get_overview_stats(start_date.as_deref(), end_date.as_deref())
                .await
            {
                let _ = sender.send(DataMessage::Overview(overview)).await;
            }

            // Load top artists
            if let Ok(artists) = db
                .get_top_artists(start_date.as_deref(), end_date.as_deref(), 50)
                .await
            {
                let _ = sender.send(DataMessage::Artists(artists)).await;
            }

            // Load top albums
            if let Ok(albums) = db
                .get_top_albums(start_date.as_deref(), end_date.as_deref(), 50)
                .await
            {
                let _ = sender.send(DataMessage::Albums(albums)).await;
            }

            // Load top tracks
            if let Ok(tracks) = db
                .get_top_tracks(start_date.as_deref(), end_date.as_deref(), 50)
                .await
            {
                let _ = sender.send(DataMessage::Tracks(tracks)).await;
            }

            // Load insights data
            let insights_data = load_insights_data(&db, start_date.as_deref(), end_date.as_deref()).await;
            if let Some(data) = insights_data {
                let _ = sender.send(DataMessage::Insights(data)).await;
            }

            // Load heatmap data
            let heatmap_data = load_heatmap_data(&db, start_date.as_deref(), end_date.as_deref()).await;
            if let Some(data) = heatmap_data {
                let _ = sender.send(DataMessage::Heatmap(data)).await;
            }

            // Load contribution data
            let contribution_data = load_contribution_data(&db, start_date.as_deref(), end_date.as_deref()).await;
            if let Some(data) = contribution_data {
                let _ = sender.send(DataMessage::Contribution(data)).await;
            }
        });
    }

    pub fn date_filter(&self) -> DateFilter {
        self.date_filter.get()
    }

    fn show_error(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(5);
        self.toast_overlay.add_toast(toast);
    }
}

async fn load_insights_data(
    db: &Database,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Option<InsightsData> {
    let streaks = db.get_listening_streaks(start_date, end_date).await.ok()?;
    let night_owl = db.get_night_owl_score(start_date, end_date).await.ok()?;
    let (skipped, total, skip_rate) = db.get_skip_rate(start_date, end_date).await.ok()?;

    Some(InsightsData {
        current_streak: streaks.current_streak,
        longest_streak: streaks.longest_streak,
        night_owl_percentage: night_owl.percentage,
        skip_rate,
        skipped_count: skipped,
        total_count: total,
    })
}

async fn load_heatmap_data(
    db: &Database,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Option<HeatmapData> {
    let heatmap = db.get_hourly_heatmap(start_date, end_date).await.ok()?;

    let mut hours = [0i64; 24];
    for (hour, count) in &heatmap.hours {
        if *hour >= 0 && *hour < 24 {
            hours[*hour as usize] = *count;
        }
    }

    Some(HeatmapData {
        hours,
        peak_hour: heatmap.peak_hour,
        peak_count: heatmap.peak_count,
    })
}

async fn load_contribution_data(
    db: &Database,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Option<ContributionData> {
    let contrib = db.get_daily_contributions(start_date, end_date).await.ok()?;

    Some(ContributionData {
        days: contrib.days,
        max_plays: contrib.max_plays,
        total_plays: contrib.total_plays,
    })
}
