//! Text input for terminals, CLI-agent prompts, and application controls.

pub mod accept_autosuggestion_keybinding_view;
pub mod autosuggestion_ignore_view;
mod soft_wrap;
mod view;

pub use view::*;
pub use warpui::text::point::Point;

use std::{cmp, ops::Range};
use warpui::AppContext;

/// A navigation key propagated from text input to its parent view.
#[derive(Debug, Clone, Copy)]
pub enum NavigationKey {
    Tab,
    ShiftTab,
    Up,
    Down,
    PageUp,
    PageDown,
    Left,
    Right,
}

pub fn init(app: &mut AppContext) {
    view::init(app);
}

trait RangeExt<T> {
    fn sorted(&self) -> (T, T);
}

impl<T: Ord + Clone> RangeExt<T> for Range<T> {
    fn sorted(&self) -> (T, T) {
        (
            cmp::min(&self.start, &self.end).clone(),
            cmp::max(&self.start, &self.end).clone(),
        )
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
pub mod tests;
