use crate::{NodeIndex, SurfaceIndex, TabStyle};
use egui::{Id, Ui, WidgetText};

/// Options for the tab bar right-click menu (Eject / Close).
pub struct TabContextMenuOptions<'a> {
    /// Localized label for the eject action.
    pub eject_label: &'a str,
    /// Localized label for the close action.
    pub close_label: &'a str,
    /// Whether eject should be offered for this tab.
    pub show_eject: bool,
    /// Whether close should be offered for this tab.
    pub show_close: bool,
}

/// Result of [`TabViewer::tab_context_menu`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TabContextMenuResponse {
    /// User chose to eject the tab into a new window.
    pub eject: bool,
    /// User chose to close the tab.
    pub close: bool,
}

/// Defines how a tab should behave and be rendered inside a [`Tree`](crate::Tree).
pub trait TabViewer {
    /// The type of tab in which you can store state to be drawn in your tabs.
    type Tab;

    /// The title to be displayed in the tab bar.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText;

    /// Actual tab content.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab);

    /// Content inside the context menu shown when the tab is right-clicked.
    ///
    /// `_surface` and `_node` specify which [`Surface`](crate::Surface) and [`Node`](crate::Node)
    /// that this particular context menu belongs to.
    fn context_menu(
        &mut self,
        _ui: &mut Ui,
        _tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
    }

    /// Tab bar right-click menu (Eject / Close). Override for custom styling.
    fn tab_context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        surface: SurfaceIndex,
        node: NodeIndex,
        options: TabContextMenuOptions<'_>,
    ) -> TabContextMenuResponse {
        use egui::Button;

        let mut response = TabContextMenuResponse::default();
        self.context_menu(ui, tab, surface, node);
        if options.show_eject && ui.add(Button::new(options.eject_label)).clicked() {
            response.eject = true;
            ui.close_menu();
        }
        if options.show_close && ui.add(Button::new(options.close_label)).clicked() {
            response.close = true;
            ui.close_menu();
        }
        response
    }

    /// Unique ID for this tab.
    ///
    /// If not implemented, uses tab title text as an ID source.
    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(self.title(tab).text())
    }

    /// Called after each tab button is shown, so you can add a tooltip, check for clicks, etc.
    fn on_tab_button(&mut self, _tab: &mut Self::Tab, _response: &egui::Response) {}

    /// Returns `true` if the user of your app should be able to close a given `_tab`.
    ///
    /// By default `true` is always returned.
    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// Returns `true` if the leaf containing `_tab` should show a collapse button.
    ///
    /// By default `true` is always returned.
    fn collapseable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// This is called when the `_tab` gets closed by the user.
    ///
    /// Returns `true` if the tab should close immediately, otherwise `false`.
    ///
    /// **Note**: if `false` is returned, [`ui`](Self::ui) will still be called once more if this
    /// tab is active.
    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// This is called when the add button is pressed.
    ///
    /// `_surface` and `_node` specify which [`Surface`](crate::Surface) and on which
    /// [`Node`](crate::Node) this particular add button was pressed.
    fn on_add(&mut self, _surface: SurfaceIndex, _node: NodeIndex) {}

    /// Content of the popup under the add button. Useful for selecting what type of tab to add.
    ///
    /// This requires that [`DockArea::show_add_buttons`](crate::DockArea::show_add_buttons) and
    /// [`DockArea::show_add_popup`](crate::DockArea::show_add_popup) are set to `true`.
    fn add_popup(&mut self, _ui: &mut Ui, _surface: SurfaceIndex, _node: NodeIndex) {}

    /// This is called every frame after [`ui`](Self::ui) is called, if the `_tab` is active.
    ///
    /// Returns `true` if the tab should be forced to close, `false` otherwise.
    ///
    /// In the event this function returns true the tab will be removed without calling `on_close`.
    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    /// Sets custom style for given tab.
    fn tab_style_override(&self, _tab: &Self::Tab, _global_style: &TabStyle) -> Option<TabStyle> {
        None
    }

    /// Specifies a tab's ability to be shown in a window.
    ///
    /// Returns `false` if this tab should never be turned into a window.
    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// Whether the tab body will be cleared with the color specified in
    /// [`TabBarStyle::bg_fill`](crate::TabBarStyle::bg_fill).
    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }

    /// Returns `true` if the horizontal and vertical scroll bars will be shown for `tab`.
    ///
    /// By default, both scroll bars are shown.
    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [true, true]
    }
}
