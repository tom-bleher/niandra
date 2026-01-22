//! HeatmapGrid widget for visualizing hourly listening patterns

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::gui::window::HeatmapData;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct HeatmapGrid {
        pub hours: RefCell<[i64; 24]>,
        pub peak_hour: Cell<i32>,
        pub peak_count: Cell<i64>,
    }

    impl Default for HeatmapGrid {
        fn default() -> Self {
            Self {
                hours: RefCell::new([0; 24]),
                peak_hour: Cell::new(0),
                peak_count: Cell::new(0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeatmapGrid {
        const NAME: &'static str = "HeatmapGrid";
        type Type = super::HeatmapGrid;
        type ParentType = gtk4::DrawingArea;
    }

    impl ObjectImpl for HeatmapGrid {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_drawing();
        }
    }

    impl WidgetImpl for HeatmapGrid {}

    impl DrawingAreaImpl for HeatmapGrid {}

    impl HeatmapGrid {
        fn setup_drawing(&self) {
            let obj = self.obj();

            obj.set_content_width(600);
            obj.set_content_height(120);

            obj.set_draw_func(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, cr, width, height| {
                    imp.draw(cr, width, height);
                }
            ));
        }

        fn draw(&self, cr: &gtk4::cairo::Context, width: i32, height: i32) {
            let hours = self.hours.borrow();
            let max_count = hours.iter().copied().max().unwrap_or(1).max(1);

            let width = width as f64;
            let height = height as f64;

            // Layout constants
            let label_height = 24.0;
            let cell_height = height - label_height - 20.0;
            let cell_width = (width - 40.0) / 24.0;
            let x_offset = 20.0;
            let y_offset = 10.0;

            // Draw cells
            for (hour, &count) in hours.iter().enumerate() {
                let x = x_offset + hour as f64 * cell_width;
                let y = y_offset;

                // Calculate color intensity (blue gradient)
                let intensity = if max_count > 0 {
                    count as f64 / max_count as f64
                } else {
                    0.0
                };

                // Background color: light gray to accent blue
                let r = 0.95 - intensity * 0.65;  // 0.95 -> 0.30
                let g = 0.95 - intensity * 0.55;  // 0.95 -> 0.40
                let b = 0.95 + intensity * 0.05;  // 0.95 -> 1.00

                cr.set_source_rgb(r, g, b);
                cr.rectangle(x + 1.0, y, cell_width - 2.0, cell_height);
                let _ = cr.fill();

                // Draw border for peak hour
                if hour as i32 == self.peak_hour.get() && count > 0 {
                    cr.set_source_rgb(0.2, 0.4, 0.8);
                    cr.set_line_width(2.0);
                    cr.rectangle(x + 1.0, y, cell_width - 2.0, cell_height);
                    let _ = cr.stroke();
                }
            }

            // Draw hour labels
            cr.set_source_rgb(0.4, 0.4, 0.4);

            // Only draw some labels to avoid crowding
            for hour in (0..24).step_by(3) {
                let x = x_offset + hour as f64 * cell_width + cell_width / 2.0;
                let y = y_offset + cell_height + 16.0;

                let label = format!("{hour:02}");

                cr.move_to(x - 8.0, y);
                let _ = cr.show_text(&label);
            }
        }
    }
}

glib::wrapper! {
    /// A custom drawing area for visualizing hourly listening patterns
    pub struct HeatmapGrid(ObjectSubclass<imp::HeatmapGrid>)
        @extends gtk4::DrawingArea, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl HeatmapGrid {
    /// Create a new heatmap grid
    #[must_use]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set heatmap data
    pub fn set_data(&self, data: &HeatmapData) {
        let imp = self.imp();
        *imp.hours.borrow_mut() = data.hours;
        imp.peak_hour.set(data.peak_hour);
        imp.peak_count.set(data.peak_count);
        self.queue_draw();
    }
}

impl Default for HeatmapGrid {
    fn default() -> Self {
        Self::new()
    }
}
