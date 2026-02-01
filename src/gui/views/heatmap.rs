//! Heatmap view showing hourly listening patterns

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;

use crate::gui::widgets::{ContributionData, ContributionGrid, HeatmapGrid};
use crate::gui::window::HeatmapData;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct HeatmapView {
        pub stack: gtk4::Stack,
        pub spinner: gtk4::Spinner,
        pub contribution_grid: RefCell<Option<ContributionGrid>>,
        pub heatmap_grid: RefCell<Option<HeatmapGrid>>,
        pub peak_hour_label: gtk4::Label,
        pub peak_count_label: gtk4::Label,
    }

    impl Default for HeatmapView {
        fn default() -> Self {
            Self {
                stack: gtk4::Stack::new(),
                spinner: gtk4::Spinner::new(),
                contribution_grid: RefCell::new(None),
                heatmap_grid: RefCell::new(None),
                peak_hour_label: gtk4::Label::new(None),
                peak_count_label: gtk4::Label::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeatmapView {
        const NAME: &'static str = "HeatmapView";
        type Type = super::HeatmapView;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for HeatmapView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for HeatmapView {}

    impl adw::subclass::prelude::BinImpl for HeatmapView {}

    impl HeatmapView {
        fn setup_ui(&self) {
            let obj = self.obj();

            self.stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

            // Loading spinner
            self.spinner.set_halign(gtk4::Align::Center);
            self.spinner.set_valign(gtk4::Align::Center);
            self.spinner.set_width_request(48);
            self.spinner.set_height_request(48);
            self.stack.add_named(&self.spinner, Some("loading"));

            // Content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let clamp = adw::Clamp::new();
            clamp.set_maximum_size(700);
            clamp.set_margin_top(24);
            clamp.set_margin_bottom(24);
            clamp.set_margin_start(12);
            clamp.set_margin_end(12);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
            content.set_halign(gtk4::Align::Center);

            // Daily Contribution Graph (GitHub-style)
            let contrib_title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            contrib_title_box.set_halign(gtk4::Align::Center);

            let contrib_title = gtk4::Label::new(Some("Daily Listening Activity"));
            contrib_title.add_css_class("title-2");
            contrib_title_box.append(&contrib_title);

            let contrib_subtitle = gtk4::Label::new(Some("Your listening history over time"));
            contrib_subtitle.add_css_class("dim-label");
            contrib_title_box.append(&contrib_subtitle);

            content.append(&contrib_title_box);

            // Contribution grid
            let contrib_frame = gtk4::Frame::new(None);
            contrib_frame.add_css_class("card");

            let contrib_grid = ContributionGrid::new();
            contrib_grid.set_margin_top(16);
            contrib_grid.set_margin_bottom(16);
            contrib_grid.set_margin_start(16);
            contrib_grid.set_margin_end(16);
            contrib_frame.set_child(Some(&contrib_grid));
            *self.contribution_grid.borrow_mut() = Some(contrib_grid);

            content.append(&contrib_frame);

            // Contribution legend
            let contrib_legend = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            contrib_legend.set_halign(gtk4::Align::Center);
            contrib_legend.set_margin_bottom(24);

            let legend_less = gtk4::Label::new(Some("Less"));
            legend_less.add_css_class("dim-label");
            legend_less.add_css_class("caption");
            contrib_legend.append(&legend_less);

            // GitHub-style color swatches
            let colors = [
                (0.922, 0.929, 0.941), // #ebedf0
                (0.608, 0.914, 0.659), // #9be9a8
                (0.251, 0.769, 0.388), // #40c463
                (0.188, 0.631, 0.306), // #30a14e
                (0.129, 0.431, 0.224), // #216e39
            ];

            for (r, g, b) in colors {
                let swatch = gtk4::DrawingArea::new();
                swatch.set_content_width(12);
                swatch.set_content_height(12);

                swatch.set_draw_func(move |_, cr, width, height| {
                    cr.set_source_rgb(r, g, b);
                    // Draw rounded rectangle
                    let radius = 2.0;
                    let w = width as f64;
                    let h = height as f64;
                    cr.new_path();
                    cr.arc(radius, radius, radius, std::f64::consts::PI, 1.5 * std::f64::consts::PI);
                    cr.arc(w - radius, radius, radius, 1.5 * std::f64::consts::PI, 2.0 * std::f64::consts::PI);
                    cr.arc(w - radius, h - radius, radius, 0.0, 0.5 * std::f64::consts::PI);
                    cr.arc(radius, h - radius, radius, 0.5 * std::f64::consts::PI, std::f64::consts::PI);
                    cr.close_path();
                    let _ = cr.fill();
                });

                contrib_legend.append(&swatch);
            }

            let legend_more = gtk4::Label::new(Some("More"));
            legend_more.add_css_class("dim-label");
            legend_more.add_css_class("caption");
            contrib_legend.append(&legend_more);

            content.append(&contrib_legend);

            // Hourly Heatmap Title
            let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            title_box.set_halign(gtk4::Align::Center);

            let title = gtk4::Label::new(Some("Hourly Listening Activity"));
            title.add_css_class("title-2");
            title_box.append(&title);

            let subtitle = gtk4::Label::new(Some("When you listen to music throughout the day"));
            subtitle.add_css_class("dim-label");
            title_box.append(&subtitle);

            content.append(&title_box);

            // Hourly Heatmap
            let grid_frame = gtk4::Frame::new(None);
            grid_frame.add_css_class("card");

            let grid = HeatmapGrid::new();
            grid.set_margin_top(16);
            grid.set_margin_bottom(16);
            grid.set_margin_start(16);
            grid.set_margin_end(16);
            grid_frame.set_child(Some(&grid));
            *self.heatmap_grid.borrow_mut() = Some(grid);

            content.append(&grid_frame);

            // Peak hour info
            let info_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 24);
            info_box.set_halign(gtk4::Align::Center);

            let peak_hour_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            let peak_hour_title = gtk4::Label::new(Some("Peak Hour"));
            peak_hour_title.add_css_class("dim-label");
            peak_hour_title.add_css_class("caption");
            self.peak_hour_label.add_css_class("title-1");
            self.peak_hour_label.set_text("—");
            peak_hour_box.append(&peak_hour_title);
            peak_hour_box.append(&self.peak_hour_label);
            info_box.append(&peak_hour_box);

            let peak_count_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            let peak_count_title = gtk4::Label::new(Some("Plays at Peak"));
            peak_count_title.add_css_class("dim-label");
            peak_count_title.add_css_class("caption");
            self.peak_count_label.add_css_class("title-1");
            self.peak_count_label.set_text("—");
            peak_count_box.append(&peak_count_title);
            peak_count_box.append(&self.peak_count_label);
            info_box.append(&peak_count_box);

            content.append(&info_box);

            // Legend
            let legend_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            legend_box.set_halign(gtk4::Align::Center);
            legend_box.set_margin_top(12);

            let legend_label = gtk4::Label::new(Some("Less"));
            legend_label.add_css_class("dim-label");
            legend_label.add_css_class("caption");
            legend_box.append(&legend_label);

            // Color swatches
            for i in 0..5 {
                let swatch = gtk4::DrawingArea::new();
                swatch.set_content_width(20);
                swatch.set_content_height(20);

                let intensity = i as f64 / 4.0;
                swatch.set_draw_func(move |_, cr, width, height| {
                    let r = 0.95 - intensity * 0.65;
                    let g = 0.95 - intensity * 0.55;
                    let b = 0.95 + intensity * 0.05;
                    cr.set_source_rgb(r, g, b);
                    cr.rectangle(0.0, 0.0, width as f64, height as f64);
                    let _ = cr.fill();
                });

                legend_box.append(&swatch);
            }

            let legend_label_more = gtk4::Label::new(Some("More"));
            legend_label_more.add_css_class("dim-label");
            legend_label_more.add_css_class("caption");
            legend_box.append(&legend_label_more);

            content.append(&legend_box);

            clamp.set_child(Some(&content));
            scrolled.set_child(Some(&clamp));
            self.stack.add_named(&scrolled, Some("content"));

            self.stack.set_visible_child_name("loading");

            obj.set_child(Some(&self.stack));
        }
    }
}

glib::wrapper! {
    /// Heatmap view showing hourly listening patterns
    pub struct HeatmapView(ObjectSubclass<imp::HeatmapView>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl HeatmapView {
    /// Create a new heatmap view
    #[must_use]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set heatmap data
    pub fn set_data(&self, data: &HeatmapData) {
        let imp = self.imp();

        if let Some(grid) = imp.heatmap_grid.borrow().as_ref() {
            grid.set_data(data);
        }

        // Format peak hour as 12-hour time
        let (hour_12, am_pm) = if data.peak_hour == 0 {
            (12, "AM")
        } else if data.peak_hour < 12 {
            (data.peak_hour, "AM")
        } else if data.peak_hour == 12 {
            (12, "PM")
        } else {
            (data.peak_hour - 12, "PM")
        };
        imp.peak_hour_label.set_text(&format!("{hour_12} {am_pm}"));

        imp.peak_count_label.set_text(&data.peak_count.to_string());

        self.set_loading(false);
    }

    /// Set contribution data for the daily grid
    pub fn set_contribution_data(&self, data: ContributionData) {
        let imp = self.imp();

        if let Some(grid) = imp.contribution_grid.borrow().as_ref() {
            grid.set_data(data);
        }
    }

    /// Set loading state
    pub fn set_loading(&self, loading: bool) {
        let imp = self.imp();

        if loading {
            imp.spinner.start();
            imp.stack.set_visible_child_name("loading");
        } else {
            imp.spinner.stop();
            imp.stack.set_visible_child_name("content");
        }
    }
}

impl Default for HeatmapView {
    fn default() -> Self {
        Self::new()
    }
}
