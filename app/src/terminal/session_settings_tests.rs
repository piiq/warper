use super::{AgentToolbarItemKind, CLIAgentToolbarChipSelection, ContextChipKind};
use serde_json::json;
use settings_value::SettingsValue;

fn custom_toolbar() -> CLIAgentToolbarChipSelection {
    CLIAgentToolbarChipSelection::Custom {
        left: vec![
            AgentToolbarItemKind::RichInput,
            AgentToolbarItemKind::ContextChip(ContextChipKind::WorkingDirectory),
            AgentToolbarItemKind::FileAttach,
        ],
        right: vec![
            AgentToolbarItemKind::ContextChip(ContextChipKind::ShellGitBranch),
            AgentToolbarItemKind::Settings,
        ],
    }
}

#[test]
fn serde_toolbar_keeps_supported_items_when_file_explorer_was_saved() {
    let saved = json!({
        "Custom": {
            "left": ["RichInput", "FileExplorer", {"ContextChip": "WorkingDirectory"}, "FileAttach"],
            "right": [{"ContextChip": "ShellGitBranch"}, "FileExplorer", "Settings"]
        }
    });
    let parsed = serde_json::from_value::<CLIAgentToolbarChipSelection>(saved).unwrap();
    assert_eq!(parsed, custom_toolbar());
}

#[test]
fn settings_file_toolbar_keeps_supported_items_when_file_explorer_was_saved() {
    let saved: toml::Value = toml::from_str(
        r#"
[custom]
left = ["rich_input", "file_explorer", { context_chip = "working_directory" }, "file_attach"]
right = [{ context_chip = "shell_git_branch" }, "file_explorer", "settings"]
"#,
    )
    .unwrap();
    let saved = serde_json::to_value(saved).unwrap();
    let parsed = CLIAgentToolbarChipSelection::from_file_value(&saved).unwrap();
    assert_eq!(parsed, custom_toolbar());
}

#[test]
fn supported_toolbar_layouts_round_trip_without_changing_file_format() {
    assert_eq!(
        custom_toolbar().to_file_value(),
        json!({
            "custom": {
                "left": ["rich_input", {"context_chip": "working_directory"}, "file_attach"],
                "right": [{"context_chip": "shell_git_branch"}, "settings"]
            }
        })
    );
    for layout in [CLIAgentToolbarChipSelection::Default, custom_toolbar()] {
        let stored = serde_json::to_value(&layout).unwrap();
        assert_eq!(
            serde_json::from_value::<CLIAgentToolbarChipSelection>(stored).unwrap(),
            layout
        );
        assert_eq!(
            CLIAgentToolbarChipSelection::from_file_value(&layout.to_file_value()).unwrap(),
            layout
        );
    }
}
