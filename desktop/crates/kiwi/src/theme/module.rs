//! Registers Cursor Dark as the active Kiwi theme.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_design::ThemeId;
use nest_theme::ThemeService;

use super::cursor_dark;

/// Module id for [`KiwiThemeModule`].
pub const KIWI_THEME_MODULE_ID: ModuleId = ModuleId("kiwi-theme");

/// Registers Cursor Dark with [`ThemeService`] and sets it active.
pub struct KiwiThemeModule;

impl Module for KiwiThemeModule {
    fn id(&self) -> ModuleId {
        KIWI_THEME_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let service = ThemeService::new();
        service.register_theme(cursor_dark::definition())?;
        service.set_active_theme(&ThemeId::from("cursor-dark"))?;
        app.register_service(service)
    }
}
