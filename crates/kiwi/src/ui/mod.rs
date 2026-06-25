mod agent;
mod mouse;
mod palette;
mod render;
mod scrollback;
mod shell;
mod status_bar;
mod tabs;

pub use mouse::map_mouse_click;
pub use mouse::mouse_interactions_enabled;
pub use palette::palette_match_at;
pub use render::draw_frame;
