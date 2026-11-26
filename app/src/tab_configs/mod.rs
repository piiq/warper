pub(crate) mod action_sidecar;
pub mod branch_picker;
pub mod metadata;
pub mod params_modal;
pub(crate) mod remove_confirmation_dialog;
pub mod repo_picker;
pub mod session_config;
pub mod session_config_modal;
pub mod session_config_rendering;
pub mod tab_config;

use warp_core::ui::theme::Fill;

pub use params_modal::{TabConfigParamsModal, TabConfigParamsModalEvent};
pub use tab_config::{
    render_tab_config, TabConfig, TabConfigError, TabConfigParam, TabConfigParamType,
};

/// Optional visual overrides for BranchPicker / RepoPicker dropdowns.
pub struct PickerStyle {
    pub width: f32,
    pub background: Option<Fill>,
}

pub mod template;
