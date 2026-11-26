use crate::app_state::{LeafContents, PaneNodeSnapshot, SplitDirection as StateSplitDirection};
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

fn is_falsey(val: &Option<bool>) -> bool {
    val.is_none_or(|v| !v)
}

/// The mode a leaf pane opens in.
///
/// Tab configurations open terminal sessions.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaneMode {
    /// A standard terminal shell session.
    #[default]
    Terminal,
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(untagged, rename_all = "lowercase")]
pub enum PaneTemplateType {
    PaneTemplate {
        cwd: PathBuf,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        commands: Vec<CommandTemplate>,
        #[serde(skip_serializing_if = "is_falsey", default)]
        is_focused: Option<bool>,
        #[serde(default)]
        pane_mode: PaneMode,
        /// Optional shell override for this pane (e.g. `"pwsh"`, `"zsh"`).
        /// Sourced from the `shell` field of a tab config pane node.
        #[serde(skip_serializing_if = "Option::is_none", default)]
        shell: Option<String>,
    },
    PaneBranchTemplate {
        split_direction: SplitDirection,
        panes: Vec<PaneTemplateType>,
    },
}

impl<'de> Deserialize<'de> for PaneTemplateType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Parse the shared node directly so invalid modes retain their specific error,
        // rather than losing it while trying each branch of an untagged enum.
        #[derive(Deserialize)]
        struct Node {
            #[serde(default, deserialize_with = "deserialize_optional_path")]
            cwd: Option<PathBuf>,
            #[serde(default)]
            commands: Vec<CommandTemplate>,
            is_focused: Option<bool>,
            #[serde(default)]
            pane_mode: PaneMode,
            shell: Option<String>,
            split_direction: Option<SplitDirection>,
            panes: Option<Vec<PaneTemplateType>>,
        }

        let node = Node::deserialize(deserializer)?;
        if let Some(cwd) = node.cwd {
            Ok(Self::PaneTemplate {
                cwd,
                commands: node.commands,
                is_focused: node.is_focused,
                pane_mode: node.pane_mode,
                shell: node.shell,
            })
        } else {
            Ok(Self::PaneBranchTemplate {
                split_direction: node
                    .split_direction
                    .ok_or_else(|| serde::de::Error::missing_field("split_direction"))?,
                panes: node
                    .panes
                    .ok_or_else(|| serde::de::Error::missing_field("panes"))?,
            })
        }
    }
}

fn deserialize_optional_path<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_path(deserializer).map(Some)
}

impl TryFrom<PaneNodeSnapshot> for PaneTemplateType {
    type Error = ();

    #[allow(clippy::unwrap_in_result)]
    fn try_from(snapshot: PaneNodeSnapshot) -> Result<Self, ()> {
        match snapshot {
            PaneNodeSnapshot::Branch(branch) => {
                let panes = branch
                    .children
                    .iter()
                    .filter_map(|(_, snapshot)| snapshot.clone().try_into().ok())
                    .collect::<Vec<PaneTemplateType>>();
                match panes.len() {
                    0 => Err(()),
                    1 => Ok(panes
                        .into_iter()
                        .next()
                        .expect("Checked that panes has 1 element")),
                    _ => Ok(Self::PaneBranchTemplate {
                        split_direction: branch.direction.into(),
                        panes,
                    }),
                }
            }
            PaneNodeSnapshot::Leaf(leaf) => match leaf.contents {
                LeafContents::Terminal(terminal) => Ok(Self::PaneTemplate {
                    cwd: PathBuf::from(terminal.cwd.unwrap_or_default()),
                    commands: Vec::new(),
                    is_focused: Some(leaf.is_focused),
                    pane_mode: PaneMode::Terminal,
                    shell: None,
                }),
                LeafContents::Settings(_) => Err(()),
            },
        }
    }
}

/// Deserializes a string that semantically represents a path, expanding ~ as
/// needed.
fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_path = String::deserialize(deserializer)?;
    Ok(PathBuf::from(shellexpand::tilde(&raw_path).into_owned()))
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Vertical,
    Horizontal,
}

impl From<StateSplitDirection> for SplitDirection {
    fn from(snapshot: StateSplitDirection) -> Self {
        match snapshot {
            StateSplitDirection::Horizontal => Self::Horizontal,
            StateSplitDirection::Vertical => Self::Vertical,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CommandTemplate {
    pub exec: String,
}

impl From<&str> for CommandTemplate {
    fn from(s: &str) -> CommandTemplate {
        CommandTemplate {
            exec: s.to_string(),
        }
    }
}
