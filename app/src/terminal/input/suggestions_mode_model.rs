use warpui::{Entity, ModelContext};

use super::InputSuggestionsMode;

/// State for terminal history and completion suggestions.
pub struct InputSuggestionsModeModel {
    mode: InputSuggestionsMode,
}

impl Default for InputSuggestionsModeModel {
    fn default() -> Self {
        Self::new()
    }
}

impl InputSuggestionsModeModel {
    pub fn new() -> Self {
        Self {
            mode: InputSuggestionsMode::Closed,
        }
    }

    pub fn mode(&self) -> &InputSuggestionsMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: InputSuggestionsMode, ctx: &mut ModelContext<Self>) {
        if self.mode != mode {
            self.mode = mode;
            ctx.emit(());
        }
    }

    pub fn is_visible(&self) -> bool {
        self.mode.is_visible()
    }
    pub fn is_closed(&self) -> bool {
        matches!(self.mode, InputSuggestionsMode::Closed)
    }
    pub fn is_history_up(&self) -> bool {
        matches!(self.mode, InputSuggestionsMode::HistoryUp { .. })
    }
    pub fn is_completion_suggestions(&self) -> bool {
        matches!(
            self.mode,
            InputSuggestionsMode::CompletionSuggestions { .. }
        )
    }
}

impl Entity for InputSuggestionsModeModel {
    type Event = ();
}
