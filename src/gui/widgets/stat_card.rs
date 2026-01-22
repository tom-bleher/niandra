//! StatCard widget for displaying key metrics

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct StatCard {
        pub title_label: RefCell<Option<gtk4::Label>>,
        pub value_label: RefCell<Option<gtk4::Label>>,
        pub subtitle_label: RefCell<Option<gtk4::Label>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StatCard {
        const NAME: &'static str = "StatCard";
        type Type = super::StatCard;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for StatCard {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for StatCard {}

    impl BoxImpl for StatCard {}

    impl StatCard {
        fn setup_ui(&self) {
            let obj = self.obj();

            obj.set_orientation(gtk4::Orientation::Vertical);
            obj.set_spacing(4);
            obj.set_halign(gtk4::Align::Center);
            obj.set_valign(gtk4::Align::Center);
            obj.add_css_class("card");
            obj.set_margin_top(12);
            obj.set_margin_bottom(12);
            obj.set_margin_start(12);
            obj.set_margin_end(12);

            // Title label
            let title = gtk4::Label::new(None);
            title.add_css_class("caption");
            title.add_css_class("dim-label");
            obj.append(&title);
            *self.title_label.borrow_mut() = Some(title);

            // Value label
            let value = gtk4::Label::new(None);
            value.add_css_class("title-1");
            obj.append(&value);
            *self.value_label.borrow_mut() = Some(value);

            // Subtitle label
            let subtitle = gtk4::Label::new(None);
            subtitle.add_css_class("caption");
            subtitle.add_css_class("dim-label");
            subtitle.set_visible(false);
            obj.append(&subtitle);
            *self.subtitle_label.borrow_mut() = Some(subtitle);
        }
    }
}

glib::wrapper! {
    /// A card widget for displaying a statistic with title, value, and optional subtitle
    pub struct StatCard(ObjectSubclass<imp::StatCard>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl StatCard {
    /// Create a new stat card with the given title
    #[must_use]
    pub fn new(title: &str) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_title(title);
        obj
    }

    /// Set the title text
    pub fn set_title(&self, title: &str) {
        if let Some(label) = self.imp().title_label.borrow().as_ref() {
            label.set_text(title);
        }
    }

    /// Set the main value text
    pub fn set_value(&self, value: &str) {
        if let Some(label) = self.imp().value_label.borrow().as_ref() {
            label.set_text(value);
        }
    }

    /// Set the subtitle text (optional, shows/hides based on content)
    pub fn set_subtitle(&self, subtitle: Option<&str>) {
        if let Some(label) = self.imp().subtitle_label.borrow().as_ref() {
            match subtitle {
                Some(text) => {
                    label.set_text(text);
                    label.set_visible(true);
                }
                None => {
                    label.set_visible(false);
                }
            }
        }
    }

    /// Set the value as a number with optional formatting
    pub fn set_value_number(&self, value: i64) {
        self.set_value(&format_number(value));
    }

    /// Set the value as hours
    pub fn set_value_hours(&self, ms: i64) {
        let hours = crate::display::format_hours(ms);
        self.set_value(&format!("{hours:.1}h"));
    }
}

impl Default for StatCard {
    fn default() -> Self {
        glib::Object::new()
    }
}

/// Format a number with thousands separators
fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    result
}
