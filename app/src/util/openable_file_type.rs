//! External application selection for terminal file links.

#[cfg(feature = "local_fs")]
use crate::util::file::external_editor::{settings::EditorChoice, Editor, EditorSettings};
#[cfg(feature = "local_fs")]
use std::path::Path;
#[cfg(feature = "local_fs")]
use warp_util::file_type::is_binary_file;

/// The application to use when opening a terminal file link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTarget {
    #[cfg(feature = "local_fs")]
    ExternalEditor(Editor),
    EnvEditor,
    SystemDefault,
    SystemGeneric,
}

#[cfg(feature = "local_fs")]
pub fn resolve_file_target(path: &Path, settings: &EditorSettings) -> FileTarget {
    resolve_file_target_with_editor_choice(path, *settings.open_file_editor)
}

#[cfg(feature = "local_fs")]
pub fn resolve_file_target_with_editor_choice(
    path: &Path,
    editor_choice: EditorChoice,
) -> FileTarget {
    if path.is_dir() || is_binary_file(path) {
        return FileTarget::SystemGeneric;
    }
    match editor_choice {
        EditorChoice::ExternalEditor(editor) => FileTarget::ExternalEditor(editor),
        EditorChoice::EnvEditor => FileTarget::EnvEditor,
        EditorChoice::SystemDefault => FileTarget::SystemDefault,
    }
}

#[cfg(all(test, feature = "local_fs"))]
#[path = "openable_file_type_tests.rs"]
mod tests;
