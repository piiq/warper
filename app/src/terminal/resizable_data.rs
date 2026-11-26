use std::collections::HashMap;

use warpui::elements::resizable_state_handle;
use warpui::elements::ResizableStateHandle;
use warpui::Entity;
use warpui::SingletonEntity;
use warpui::WindowId;

use crate::app_state::WindowSnapshot;

pub const DEFAULT_UNIVERSAL_SEARCH_WIDTH: f32 = 700.;
pub const DEFAULT_SETTINGS_PANEL_WIDTH: f32 = 194.;
/// Resizable search and settings regions.
pub enum ModalType {
    UniversalSearchWidth,
    SettingsPanelWidth,
}

/// Search and settings width handles for one window.
pub struct ModalSizes {
    pub universal_search_width: ResizableStateHandle,
    pub settings_panel_width: ResizableStateHandle,
}

impl ModalSizes {
    /// Restores the search width and uses the default settings width.
    pub fn from_restored(window_snapshot: &WindowSnapshot) -> Self {
        let universal_search_width = window_snapshot
            .universal_search_width
            .unwrap_or(DEFAULT_UNIVERSAL_SEARCH_WIDTH);
        let settings_panel_width = DEFAULT_SETTINGS_PANEL_WIDTH;

        Self {
            universal_search_width: resizable_state_handle(universal_search_width),
            settings_panel_width: resizable_state_handle(settings_panel_width),
        }
    }

    pub fn get_resizable_state_handle(&self, modal: ModalType) -> ResizableStateHandle {
        match modal {
            ModalType::UniversalSearchWidth => self.universal_search_width.clone(),
            ModalType::SettingsPanelWidth => self.settings_panel_width.clone(),
        }
    }
}

impl Default for ModalSizes {
    fn default() -> Self {
        Self {
            universal_search_width: resizable_state_handle(DEFAULT_UNIVERSAL_SEARCH_WIDTH),
            settings_panel_width: resizable_state_handle(DEFAULT_SETTINGS_PANEL_WIDTH),
        }
    }
}

/// Search and settings width handles keyed by window.
#[derive(Default)]
pub struct ResizableData {
    sizes_per_window: HashMap<WindowId, ModalSizes>,
}

pub enum ResizableDataEvent {}

impl ResizableData {
    pub fn insert(&mut self, window_id: WindowId, modal_sizes: ModalSizes) {
        self.sizes_per_window.insert(window_id, modal_sizes);
    }

    pub fn get_handle(
        &self,
        window_id: WindowId,
        modal: ModalType,
    ) -> Option<ResizableStateHandle> {
        self.sizes_per_window
            .get(&window_id)
            .map(|modal_sizes| modal_sizes.get_resizable_state_handle(modal))
    }

    pub fn get_all_handles(&self, window_id: WindowId) -> Option<&ModalSizes> {
        self.sizes_per_window.get(&window_id)
    }
}

impl Entity for ResizableData {
    type Event = ResizableDataEvent;
}

impl SingletonEntity for ResizableData {}
