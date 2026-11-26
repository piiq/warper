use super::*;

#[test]
fn markdown_and_code_links_use_the_selected_external_editor() {
    for filename in ["README.md", "src/main.rs", "notes.txt"] {
        assert_eq!(
            resolve_file_target_with_editor_choice(
                Path::new(filename),
                EditorChoice::ExternalEditor(Editor::VSCode)
            ),
            FileTarget::ExternalEditor(Editor::VSCode),
        );
    }
}

#[test]
fn environment_editor_and_system_default_remain_available() {
    assert_eq!(
        resolve_file_target_with_editor_choice(Path::new("main.rs"), EditorChoice::EnvEditor),
        FileTarget::EnvEditor
    );
    assert_eq!(
        resolve_file_target_with_editor_choice(Path::new("main.rs"), EditorChoice::SystemDefault),
        FileTarget::SystemDefault
    );
}

#[test]
fn binary_links_use_the_system_application() {
    assert_eq!(
        resolve_file_target_with_editor_choice(
            Path::new("image.png"),
            EditorChoice::ExternalEditor(Editor::VSCode)
        ),
        FileTarget::SystemGeneric
    );
}
