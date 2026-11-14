use serde::{Deserialize, Serialize};

use crate::features::FeatureFlag;
use crate::ui_components::icons::Icon;
use crate::workspace::tab_settings::TabSettings;

use warpui::{AppContext, SingletonEntity};

/// A configurable item in the vertical tabs header toolbar.
///
/// Each variant represents a panel toggle button that can be placed on either
/// the left or right side of the toolbar. The side determines which side of the
/// main content area the panel opens on.
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(rename_all = "snake_case")]
pub enum HeaderToolbarItemKind {
    TabsPanel,
    ToolsPanel,
}

impl HeaderToolbarItemKind {
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::TabsPanel => "Tabs Panel",
            Self::ToolsPanel => "Tools Panel",
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            Self::TabsPanel => Icon::Menu,
            Self::ToolsPanel => Icon::Tool2,
        }
    }

    /// Whether this item is supported by the current layout and feature flags.
    pub fn is_supported(&self, app: &AppContext) -> bool {
        match self {
            Self::TabsPanel => {
                FeatureFlag::VerticalTabs.is_enabled()
                    && *TabSettings::as_ref(app).use_vertical_tabs
            }
            Self::ToolsPanel => true,
        }
    }

    /// Whether this item should be shown in the toolbar.
    pub fn is_available(&self, app: &AppContext) -> bool {
        self.is_supported(app)
    }

    /// Whether this item opens a side panel (as opposed to replacing the content
    /// area or opening a popover).
    pub fn is_panel(&self) -> bool {
        matches!(self, Self::TabsPanel | Self::ToolsPanel)
    }

    pub fn default_left() -> Vec<Self> {
        vec![Self::TabsPanel, Self::ToolsPanel]
    }

    pub fn default_right() -> Vec<Self> {
        vec![]
    }

    /// All toolbar item variants (availability filtering is done at the call site).
    pub fn all_items() -> Vec<Self> {
        vec![Self::TabsPanel, Self::ToolsPanel]
    }
}
