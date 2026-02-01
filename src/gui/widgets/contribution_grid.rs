//! ContributionGrid widget for visualizing daily listening patterns (GitHub-style)

use chrono::Datelike;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// Daily contribution data
#[derive(Debug, Clone, Default)]
pub struct ContributionData {
    /// Map of date string (YYYY-MM-DD) to play count
    pub days: HashMap<String, i64>,
    /// Maximum plays in a single day
    pub max_plays: i64,
    /// Total plays in the period
    pub total_plays: i64,
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ContributionGrid {
        pub data: RefCell<ContributionData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContributionGrid {
        const NAME: &'static str = "ContributionGrid";
        type Type = super::ContributionGrid;
        type ParentType = gtk4::DrawingArea;
    }

    impl ObjectImpl for ContributionGrid {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_drawing();
        }
    }

    impl WidgetImpl for ContributionGrid {}

    impl DrawingAreaImpl for ContributionGrid {}

    impl ContributionGrid {
        fn setup_drawing(&self) {
            let obj = self.obj();

            obj.set_content_width(722);  // 52 weeks * 13px + padding
            obj.set_content_height(120); // 7 days * 13px + labels

            obj.set_draw_func(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, cr, width, height| {
                    imp.draw(cr, width, height);
                }
            ));
        }

        fn draw(&self, cr: &gtk4::cairo::Context, width: i32, height: i32) {
            let data = self.data.borrow();

            let width = width as f64;
            let height = height as f64;

            // Layout constants
            let cell_size = 11.0;
            let cell_gap = 2.0;
            let cell_total = cell_size + cell_gap;
            let label_width = 28.0;
            let label_height = 16.0;
            let x_offset = label_width;
            let y_offset = label_height;

            // Calculate how many weeks we can fit
            let available_width = width - label_width - 10.0;
            let num_weeks = ((available_width / cell_total) as i32).min(53);

            // Get today's date and calculate the grid
            let today = chrono::Local::now().date_naive();
            let today_weekday = today.weekday().num_days_from_sunday() as i32;

            // Draw month labels at the top
            cr.set_source_rgb(0.5, 0.5, 0.5);
            cr.select_font_face("Sans", gtk4::cairo::FontSlant::Normal, gtk4::cairo::FontWeight::Normal);
            cr.set_font_size(10.0);

            let mut last_month = -1i32;
            for week in 0..num_weeks {
                let days_back = (num_weeks - 1 - week) * 7 + today_weekday;
                if let Some(date) = today.checked_sub_days(chrono::Days::new(days_back as u64)) {
                    let month = date.month() as i32;
                    if month != last_month {
                        let month_name = match month {
                            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                            _ => "",
                        };
                        let x = x_offset + week as f64 * cell_total;
                        cr.move_to(x, 10.0);
                        let _ = cr.show_text(month_name);
                        last_month = month;
                    }
                }
            }

            // Draw day labels on the left
            let day_labels = ["", "Mon", "", "Wed", "", "Fri", ""];
            for (day, label) in day_labels.iter().enumerate() {
                if !label.is_empty() {
                    let y = y_offset + day as f64 * cell_total + cell_size;
                    cr.move_to(0.0, y);
                    let _ = cr.show_text(label);
                }
            }

            // Draw the contribution squares
            for week in 0..num_weeks {
                for day in 0..7 {
                    // Calculate the date for this cell
                    let days_back = (num_weeks - 1 - week) * 7 + (6 - day) + (today_weekday - 6);
                    let days_back = if days_back < 0 { continue } else { days_back as u64 };

                    let Some(date) = today.checked_sub_days(chrono::Days::new(days_back)) else {
                        continue;
                    };

                    // Don't draw future dates
                    if date > today {
                        continue;
                    }

                    let date_str = date.format("%Y-%m-%d").to_string();
                    let count = data.days.get(&date_str).copied().unwrap_or(0);

                    let x = x_offset + week as f64 * cell_total;
                    let y = y_offset + day as f64 * cell_total;

                    // Calculate color intensity (GitHub-style green gradient)
                    let intensity = if data.max_plays > 0 && count > 0 {
                        // Use logarithmic scale for better distribution
                        let normalized = (count as f64).ln() / (data.max_plays as f64).ln();
                        normalized.clamp(0.0, 1.0)
                    } else {
                        0.0
                    };

                    // GitHub-style colors: #ebedf0 (empty) -> #9be9a8 -> #40c463 -> #30a14e -> #216e39
                    let (r, g, b) = if count == 0 {
                        (0.922, 0.929, 0.941) // #ebedf0
                    } else if intensity < 0.25 {
                        (0.608, 0.914, 0.659) // #9be9a8
                    } else if intensity < 0.5 {
                        (0.251, 0.769, 0.388) // #40c463
                    } else if intensity < 0.75 {
                        (0.188, 0.631, 0.306) // #30a14e
                    } else {
                        (0.129, 0.431, 0.224) // #216e39
                    };

                    cr.set_source_rgb(r, g, b);

                    // Draw rounded rectangle
                    let radius = 2.0;
                    cr.new_path();
                    cr.arc(x + radius, y + radius, radius, std::f64::consts::PI, 1.5 * std::f64::consts::PI);
                    cr.arc(x + cell_size - radius, y + radius, radius, 1.5 * std::f64::consts::PI, 2.0 * std::f64::consts::PI);
                    cr.arc(x + cell_size - radius, y + cell_size - radius, radius, 0.0, 0.5 * std::f64::consts::PI);
                    cr.arc(x + radius, y + cell_size - radius, radius, 0.5 * std::f64::consts::PI, std::f64::consts::PI);
                    cr.close_path();
                    let _ = cr.fill();
                }
            }
        }
    }
}

glib::wrapper! {
    /// A GitHub-style contribution grid showing daily listening activity
    pub struct ContributionGrid(ObjectSubclass<imp::ContributionGrid>)
        @extends gtk4::DrawingArea, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl ContributionGrid {
    /// Create a new contribution grid
    #[must_use]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set contribution data
    pub fn set_data(&self, data: ContributionData) {
        *self.imp().data.borrow_mut() = data;
        self.queue_draw();
    }
}

impl Default for ContributionGrid {
    fn default() -> Self {
        Self::new()
    }
}
