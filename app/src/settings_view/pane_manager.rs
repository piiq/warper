use crate::pane_group::SettingsPane;
use crate::{
    pane_group::{PaneContent, PaneId},
    PaneViewLocator,
};
use std::collections::HashMap;
use warpui::{Entity, EntityId, ModelContext, SingletonEntity, ViewHandle, WindowId};

use super::SettingsView;
struct SettingsPaneData {
    locator: Option<PaneViewLocator>,
    settings_view: ViewHandle<SettingsView>,
}

/// Singleton model to manage state of settings panes across multiple windows
/// (where only one settings pane can exist per window). Specifically:
/// - Maintains settings view handles to preserve state when panes are hidden
/// - Tracks currently open settings panes and their location
#[derive(Default)]
pub struct SettingsPaneManager {
    panes: HashMap<WindowId, SettingsPaneData>,
}

impl SettingsPaneManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn settings_view(&self, window_id: WindowId) -> ViewHandle<SettingsView> {
        self.panes
            .get(&window_id)
            .expect("Window should have corresponding settings view")
            .settings_view
            .clone()
    }

    pub fn register_view(&mut self, window_id: WindowId, view: ViewHandle<SettingsView>) {
        if let Some(data) = self.panes.get_mut(&window_id) {
            data.settings_view = view;
        } else {
            self.panes.insert(
                window_id,
                SettingsPaneData {
                    locator: None,
                    settings_view: view,
                },
            );
        }
    }

    pub fn find_pane(&self, window_id: WindowId) -> Option<PaneViewLocator> {
        self.panes.get(&window_id).and_then(|data| data.locator)
    }

    /// Moves the shared settings view and its open-pane location with a transferred tab.
    pub fn transfer_pane(
        &mut self,
        source_window_id: WindowId,
        destination_window_id: WindowId,
        pane_group_id: EntityId,
    ) -> Option<ViewHandle<SettingsView>> {
        let source = self.panes.get_mut(&source_window_id)?;
        let locator = source
            .locator
            .filter(|locator| locator.pane_group_id == pane_group_id)?;
        source.locator = None;
        let settings_view = source.settings_view.clone();

        let destination = self
            .panes
            .get_mut(&destination_window_id)
            .expect("The destination workspace has registered its settings view");
        destination.settings_view = settings_view.clone();
        destination.locator = Some(locator);
        Some(settings_view)
    }

    pub fn register_pane(
        &mut self,
        pane: &SettingsPane,
        pane_group_id: EntityId,
        window_id: WindowId,
        _ctx: &mut ModelContext<Self>,
    ) {
        if let Some(data) = self.panes.get_mut(&window_id) {
            data.locator = Some(PaneViewLocator {
                pane_group_id,
                pane_id: pane.id(),
            });
        } else {
            log::warn!("Settings view should already exist for settings pane");
        }
    }

    pub fn deregister_pane(
        &mut self,
        window_id: &WindowId,
        pane_group_id: EntityId,
        pane_id: PaneId,
        _ctx: &mut ModelContext<Self>,
    ) {
        if let Some(data) = self.panes.get_mut(window_id) {
            let locator = PaneViewLocator {
                pane_group_id,
                pane_id,
            };
            if data.locator == Some(locator) {
                data.locator = None;
            }
        }
    }
}

impl Entity for SettingsPaneManager {
    type Event = ();
}

/// Mark SettingsPaneManager as global application state.
impl SingletonEntity for SettingsPaneManager {}
