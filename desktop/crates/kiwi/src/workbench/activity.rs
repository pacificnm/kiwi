//! Activity bar — selection enum and icon column.

use egui::Ui;
use nest_icon::{Icon, icons};

/// Activity bar selection — controls left sidebar content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Activity {
    /// File explorer tree.
    #[default]
    Explorer,
    /// Project search.
    Search,
    /// Source control.
    SourceControl,
    /// Issues tracker.
    Issues,
    /// Task list.
    Tasks,
    /// AI agent tools.
    Agent,
    /// Utility tools.
    Tools,
    /// Extensions marketplace.
    Extensions,
}

impl Activity {
    /// All activity bar items in display order.
    pub const ALL: [Self; 8] = [
        Self::Explorer,
        Self::Search,
        Self::SourceControl,
        Self::Issues,
        Self::Tasks,
        Self::Agent,
        Self::Tools,
        Self::Extensions,
    ];

    /// Short label for tooltips and sidebar headings.
    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Search => "Search",
            Self::SourceControl => "Source Control",
            Self::Issues => "Issues",
            Self::Tasks => "Tasks",
            Self::Agent => "Agent",
            Self::Tools => "Tools",
            Self::Extensions => "Extensions",
        }
    }

    /// Font Awesome icon for this activity.
    pub fn icon(self) -> Icon {
        match self {
            Self::Explorer => Icon::solid(icons::solid::FOLDER),
            Self::Search => Icon::MAGNIFYING_GLASS,
            Self::SourceControl => Icon::solid(icons::solid::LINK),
            Self::Issues => Icon::solid(icons::solid::CIRCLE_EXCLAMATION),
            Self::Tasks => Icon::CHECK,
            Self::Agent => Icon::solid(icons::solid::USER),
            Self::Tools => Icon::solid(icons::solid::BARS),
            Self::Extensions => Icon::solid(icons::solid::PLUS),
        }
    }
}

pub(crate) const ACTIVITY_BAR_WIDTH: f32 = 48.0;
const ACTIVITY_ICON_SIZE: f32 = 16.0;

/// Renders the full-height activity icon column.
pub fn activity_bar(ui: &mut Ui, activity: &mut Activity) {
    ui.set_min_height(ui.available_height());
    ui.set_width(ACTIVITY_BAR_WIDTH);

    ui.vertical(|ui| {
        ui.add_space(8.0);
        for item in Activity::ALL {
            activity_button(ui, activity, item);
        }
        ui.allocate_space(egui::vec2(0.0, ui.available_height().max(0.0)));
        if icon_button(ui, Icon::GEAR, "Settings", false).clicked() {
            // placeholder
        }
        ui.add_space(8.0);
    });
}

fn activity_button(ui: &mut Ui, activity: &mut Activity, item: Activity) {
    let selected = *activity == item;
    if icon_button(ui, item.icon(), item.tooltip(), selected).clicked() {
        *activity = item;
    }
}

fn icon_button(ui: &mut Ui, icon: Icon, tooltip: &str, selected: bool) -> egui::Response {
    let size = egui::vec2(ACTIVITY_BAR_WIDTH, 36.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if selected || response.hovered() {
        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().widgets.hovered.bg_fill
        };
        ui.painter()
            .rect_filled(rect.shrink(4.0), 6.0, fill);
    }

    let color = if selected {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().weak_text_color()
    };

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon.glyph().to_string(),
        egui::FontId::new(ACTIVITY_ICON_SIZE, icon.style().font_family()),
        color,
    );

    response.on_hover_text(tooltip)
}
