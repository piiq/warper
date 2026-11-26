//! Grid tooltips for the terminal view

use pathfinder_geometry::vector::vec2f;

use warpui::{
    elements::{
        ChildAnchor, Dismiss, MouseStateHandle, OffsetPositioning, PositionedElementAnchor,
        PositionedElementOffsetBounds, Stack,
    },
    AppContext, Element, EventContext,
};

use super::{TerminalAction, TerminalView};
use crate::util::tooltips::TooltipLink;
use crate::{
    appearance::Appearance,
    terminal::{
        links::directly_open_link_keybinding_string,
        model::{ObfuscateSecrets, Secret},
        safe_mode_settings::get_secret_obfuscation_mode,
        view::SecretTooltip,
        TerminalModel,
    },
};

struct GridTooltipLink {
    text: String,
    action: TerminalAction,
    /// Optional detail text to show after the link.
    detail: Option<String>,
    mouse_state: MouseStateHandle,
}

impl TerminalView {
    /// Renders the link and/or secrets tooltips on top of the grid
    /// Expects at least one of the two tooltips to be visible.
    // Unused variables allowed when no local filesystem as the `app` argument
    // is unused.
    #[cfg_attr(not(feature = "local_fs"), allow(unused_variables))]
    pub(super) fn render_grid_tooltip(
        &self,
        stack: &mut Stack,
        model: &TerminalModel,
        appearance: &Appearance,
        app: &AppContext,
    ) {
        let mut element_id = "terminal_view:first_cell_in_link".to_string();
        let mut links = vec![];

        if let Some(open_secret_tooltip) = &self.open_secret_tool_tip {
            match open_secret_tooltip {
                SecretTooltip::Grid { tooltip } => {
                    let handle = *tooltip;

                    // Position the tooltip above the first cell in the secret.
                    element_id = format!(
                        "terminal_view:first_cell_in_secret_{}",
                        handle.get_inner().id()
                    );

                    if matches!(get_secret_obfuscation_mode(app), ObfuscateSecrets::Yes) {
                        let is_redacted = model
                            .secret_from_handle(tooltip)
                            .is_some_and(Secret::is_obfuscated);

                        if is_redacted {
                            links.push(GridTooltipLink {
                                text: "Reveal secret".to_string(),
                                action: TerminalAction::ToggleGridSecret {
                                    handle,
                                    show_secret: true,
                                },
                                mouse_state: self.mouse_states.toggle_secrets_tooltip.clone(),
                                detail: None,
                            });
                        } else {
                            links.push(GridTooltipLink {
                                text: "Hide secret".to_string(),
                                action: TerminalAction::ToggleGridSecret {
                                    handle,
                                    show_secret: false,
                                },
                                mouse_state: self.mouse_states.toggle_secrets_tooltip.clone(),
                                detail: None,
                            })
                        }
                    }

                    links.push(GridTooltipLink {
                        text: "Copy secret".to_string(),
                        action: TerminalAction::CopyGridSecret(handle),
                        mouse_state: self.mouse_states.copy_secrets_tooltip.clone(),
                        detail: None,
                    });
                }
            }
        }

        #[cfg_attr(not(feature = "local_fs"), allow(unused_mut))]
        if let Some(link) = &self.open_grid_link_tool_tip {
            let modifier = directly_open_link_keybinding_string();
            let detail = Some(format!("[{modifier} Click]"));

            links.push(GridTooltipLink {
                text: link.tooltip_text().to_owned(),
                action: TerminalAction::OpenGridLink(link.clone()),
                mouse_state: self.mouse_states.grid_link_tooltip.clone(),
                detail,
            });
        }

        #[cfg_attr(not(feature = "local_fs"), allow(unused_mut))]
        if let Some(tooltip_info) = &self.open_rich_content_link_tool_tip {
            element_id = tooltip_info.position_id.to_owned();
            let modifier_string = directly_open_link_keybinding_string();
            let detail = Some(format!("[{modifier_string} Click]"));

            links.push(GridTooltipLink {
                text: tooltip_info.link.tooltip_text().to_owned(),
                action: TerminalAction::OpenRichContentLink(tooltip_info.link.clone()),
                mouse_state: self.mouse_states.rich_content_link_tooltip.clone(),
                detail,
            });
        }

        stack.add_positioned_overlay_child(
            render_tooltip(links, appearance),
            OffsetPositioning::offset_from_save_position_element(
                element_id,
                // Add a small buffer between the tooltip and the top of the cell.
                vec2f(0., -2.),
                PositionedElementOffsetBounds::ParentByPosition,
                PositionedElementAnchor::TopLeft,
                ChildAnchor::BottomLeft,
            ),
        );
    }
}

fn render_tooltip(
    tooltip_links: impl IntoIterator<Item = GridTooltipLink>,
    appearance: &Appearance,
) -> Box<dyn Element> {
    // Convert GridTooltipLink to shared TooltipLink
    let shared_links = tooltip_links.into_iter().map(|link| {
        let action = link.action;
        TooltipLink {
            text: link.text,
            on_click: move |ctx: &mut EventContext| {
                ctx.dispatch_typed_action(action.clone());
            },
            detail: link.detail,
            mouse_state: link.mouse_state,
        }
    });

    let tooltip_content = crate::util::tooltips::render_tooltip(shared_links, appearance);

    Dismiss::new(tooltip_content)
        .on_dismiss(|ctx, _app| {
            ctx.dispatch_typed_action(TerminalAction::MaybeDismissToolTip {
                from_keybinding: false,
            })
        })
        .finish()
}
