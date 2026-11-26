use crate::search::command_palette::new_session::{NewSessionOption, NewSessionOptionId};
use crate::search::mixer::SearchMixer;
use crate::util::bindings::CommandBinding;
use crate::workspace::PaneViewLocator;
use std::sync::Arc;
use strum_macros::IntoStaticStr;
use warpui::{keymap::BindingId, WindowId};

pub type CommandPaletteMixer = SearchMixer<CommandPaletteItemAction>;

#[derive(Clone, Debug)]
pub enum CommandPaletteItemAction {
    AcceptBinding {
        binding: Arc<CommandBinding>,
    },
    NavigateToSession {
        pane_view_locator: PaneViewLocator,
        window_id: WindowId,
    },
    NewSession {
        source: Arc<NewSessionOption>,
    },
    NoOp,
}
impl CommandPaletteItemAction {
    pub fn to_summary(&self) -> ItemSummary {
        match self {
            Self::AcceptBinding { binding } => ItemSummary::Action {
                binding_id: binding.id,
            },
            Self::NavigateToSession {
                pane_view_locator, ..
            } => ItemSummary::Session {
                pane_view_locator: *pane_view_locator,
            },
            Self::NewSession { source } => ItemSummary::NewSession {
                id: source.id().clone(),
            },
            Self::NoOp => ItemSummary::NoOp,
        }
    }
    pub fn result_type(&self) -> &'static str {
        self.to_summary().into()
    }
}

#[derive(Clone, Debug, PartialEq, IntoStaticStr)]
pub enum ItemSummary {
    Action { binding_id: BindingId },
    Session { pane_view_locator: PaneViewLocator },
    NewSession { id: NewSessionOptionId },
    NoOp,
}
