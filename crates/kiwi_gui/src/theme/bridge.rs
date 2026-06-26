//! Map [`ThemePalette`] semantic roles to egui styling (SPEC-023).

use std::collections::HashMap;

use egui::{Color32, FontId, Stroke, TextStyle, Visuals};
use kiwi_core::config::GuiSettings;
use kiwi_core::theme::{ResolvedColor, RoleStyle, SemanticRole, ThemePalette};

/// egui chrome plus semantic role colors for domain panels.
#[derive(Debug, Clone)]
pub struct GuiTheme {
    pub visuals: Visuals,
    semantic: HashMap<SemanticRole, Color32>,
    pub font_size: f32,
}

impl GuiTheme {
    #[must_use]
    pub fn from_palette(palette: &ThemePalette, gui: &GuiSettings) -> Self {
        let semantic = build_semantic_map(palette);
        let bg = role_color(&semantic, SemanticRole::Bg);
        let dark_mode = is_dark_theme(&palette.name, bg);
        let mut visuals = if dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        apply_chrome(&mut visuals, &semantic, dark_mode);
        Self {
            visuals,
            semantic,
            font_size: gui.font_size,
        }
    }

    #[must_use]
    pub fn role(&self, role: SemanticRole) -> Color32 {
        self.semantic
            .get(&role)
            .copied()
            .unwrap_or_else(|| role_color(&self.semantic, SemanticRole::Fg))
    }

    pub fn apply_to_context(&self, ctx: &egui::Context) {
        ctx.set_visuals(self.visuals.clone());
        let scaled = self.font_size * ctx.pixels_per_point();
        ctx.style_mut(|style| {
            style
                .text_styles
                .insert(TextStyle::Body, FontId::proportional(scaled));
            style
                .text_styles
                .insert(TextStyle::Button, FontId::proportional(scaled));
            style
                .text_styles
                .insert(TextStyle::Heading, FontId::proportional(scaled * 1.25));
            style
                .text_styles
                .insert(TextStyle::Monospace, FontId::monospace(scaled));
            style
                .text_styles
                .insert(TextStyle::Small, FontId::proportional(scaled * 0.85));
        });
    }
}

fn build_semantic_map(palette: &ThemePalette) -> HashMap<SemanticRole, Color32> {
    let fallback_fg = palette_color(palette, SemanticRole::Fg);
    SemanticRole::ALL
        .into_iter()
        .map(|role| {
            let style = palette.get(role);
            let color = role_style_color(style).unwrap_or(fallback_fg);
            (role, color)
        })
        .collect()
}

fn apply_chrome(visuals: &mut Visuals, semantic: &HashMap<SemanticRole, Color32>, dark_mode: bool) {
    let bg = role_color(semantic, SemanticRole::Bg);
    let fg = role_color(semantic, SemanticRole::Fg);
    let border = role_color(semantic, SemanticRole::Border);
    let accent = role_color(semantic, SemanticRole::Accent);
    let muted = role_color(semantic, SemanticRole::Muted);
    let selection = role_color(semantic, SemanticRole::Selection);

    visuals.dark_mode = dark_mode;
    visuals.panel_fill = bg;
    visuals.window_fill = bg;
    visuals.extreme_bg_color = adjust_brightness(bg, if dark_mode { -0.08 } else { 0.06 });
    visuals.faint_bg_color = blend(bg, muted, 0.35);
    visuals.code_bg_color = visuals.extreme_bg_color;
    visuals.override_text_color = Some(fg);
    visuals.window_stroke = Stroke::new(1.0, border);
    visuals.hyperlink_color = accent;
    visuals.selection.bg_fill = selection;
    visuals.selection.stroke = Stroke::new(1.0, accent);
    visuals.warn_fg_color = role_color(semantic, SemanticRole::AgentWarning);
    visuals.error_fg_color = role_color(semantic, SemanticRole::AgentError);

    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        apply_widget_visuals(widget, fg, bg, border, accent, muted, selection);
    }
}

fn apply_widget_visuals(
    widget: &mut egui::style::WidgetVisuals,
    fg: Color32,
    bg: Color32,
    border: Color32,
    _accent: Color32,
    muted: Color32,
    _selection: Color32,
) {
    widget.fg_stroke = Stroke::new(1.0, fg);
    widget.bg_stroke = Stroke::new(1.0, border);
    widget.bg_fill = blend(bg, muted, 0.12);
    widget.weak_bg_fill = blend(bg, muted, 0.2);
}

fn role_color(semantic: &HashMap<SemanticRole, Color32>, role: SemanticRole) -> Color32 {
    semantic
        .get(&role)
        .copied()
        .unwrap_or_else(|| Color32::from_gray(128))
}

fn palette_color(palette: &ThemePalette, role: SemanticRole) -> Color32 {
    role_style_color(palette.get(role)).unwrap_or_else(|| Color32::from_gray(128))
}

fn role_style_color(style: RoleStyle) -> Option<Color32> {
    style
        .fg
        .map(resolved_to_color32)
        .or_else(|| style.bg.map(resolved_to_color32))
}

fn resolved_to_color32(color: ResolvedColor) -> Color32 {
    match color {
        ResolvedColor::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
        ResolvedColor::Indexed(index) => ansi_index_to_color32(index),
    }
}

fn ansi_index_to_color32(index: u8) -> Color32 {
    // Standard 16-color ANSI palette for indexed theme values.
    const ANSI16: [Color32; 16] = [
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(128, 0, 0),
        Color32::from_rgb(0, 128, 0),
        Color32::from_rgb(128, 128, 0),
        Color32::from_rgb(0, 0, 128),
        Color32::from_rgb(128, 0, 128),
        Color32::from_rgb(0, 128, 128),
        Color32::from_rgb(192, 192, 192),
        Color32::from_rgb(128, 128, 128),
        Color32::from_rgb(255, 0, 0),
        Color32::from_rgb(0, 255, 0),
        Color32::from_rgb(255, 255, 0),
        Color32::from_rgb(0, 0, 255),
        Color32::from_rgb(255, 0, 255),
        Color32::from_rgb(0, 255, 255),
        Color32::from_rgb(255, 255, 255),
    ];
    ANSI16
        .get(usize::from(index % 16))
        .copied()
        .unwrap_or(Color32::from_gray(128))
}

fn is_dark_theme(name: &str, bg: Color32) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower == "kiwi-light" || lower.ends_with("-light") || lower.ends_with("-latte") {
        return false;
    }
    relative_luminance(bg) < 0.4
}

fn relative_luminance(color: Color32) -> f32 {
    let r = f32::from(color.r()) / 255.0;
    let g = f32::from(color.g()) / 255.0;
    let b = f32::from(color.b()) / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn adjust_brightness(color: Color32, delta: f32) -> Color32 {
    blend(
        color,
        if delta < 0.0 {
            Color32::BLACK
        } else {
            Color32::WHITE
        },
        delta.abs(),
    )
}

fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| ((1.0 - t) * f32::from(x) + t * f32::from(y)).round() as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}

#[cfg(test)]
mod tests {
    use egui::Color32;
    use kiwi_core::config::{GuiSettings, ThemeSettings};
    use kiwi_core::theme::{load_theme_with_capabilities, SemanticRole, TerminalCapabilities};

    use super::{is_dark_theme, GuiTheme};

    fn load(name: &str) -> kiwi_core::theme::ThemePalette {
        let settings = ThemeSettings {
            name: name.to_string(),
            custom: None,
        };
        load_theme_with_capabilities(&settings, TerminalCapabilities::TrueColor).expect("theme")
    }

    #[test]
    fn kiwi_dark_git_modified_matches_toml_hex() {
        let palette = load("kiwi-dark");
        let theme = GuiTheme::from_palette(&palette, &GuiSettings::default());
        assert_eq!(
            theme.role(SemanticRole::GitModified),
            Color32::from_rgb(224, 175, 104)
        );
    }

    #[test]
    fn kiwi_dark_panel_fill_matches_bg_role() {
        let palette = load("kiwi-dark");
        let theme = GuiTheme::from_palette(&palette, &GuiSettings::default());
        assert_eq!(theme.visuals.panel_fill, theme.role(SemanticRole::Bg));
        assert!(theme.visuals.dark_mode);
    }

    #[test]
    fn kiwi_light_uses_light_base_visuals() {
        let palette = load("kiwi-light");
        let theme = GuiTheme::from_palette(&palette, &GuiSettings::default());
        assert!(!theme.visuals.dark_mode);
        assert_eq!(theme.role(SemanticRole::Fg), Color32::from_rgb(52, 59, 88));
    }

    #[test]
    fn custom_font_size_from_gui_settings() {
        let palette = load("kiwi-dark");
        let gui = GuiSettings { font_size: 18.0 };
        let theme = GuiTheme::from_palette(&palette, &gui);
        assert!((theme.font_size - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn is_dark_heuristic_recognizes_light_suffix() {
        assert!(!is_dark_theme(
            "catppuccin-latte",
            Color32::from_rgb(239, 241, 245)
        ));
        assert!(is_dark_theme(
            "catppuccin-mocha",
            Color32::from_rgb(30, 30, 46)
        ));
    }

    #[test]
    fn semantic_map_includes_issue_and_agent_roles() {
        let palette = load("kiwi-dark");
        let theme = GuiTheme::from_palette(&palette, &GuiSettings::default());
        assert_ne!(
            theme.role(SemanticRole::IssueOpen),
            theme.role(SemanticRole::IssueClosed)
        );
        assert_ne!(
            theme.role(SemanticRole::AgentSuccess),
            theme.role(SemanticRole::AgentError)
        );
    }
}
