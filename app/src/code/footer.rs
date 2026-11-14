use std::path::{Path, PathBuf};

use lsp::{LanguageId, LspManagerModel, LspServerModel, LspState as LspModelState};

use pathfinder_color::ColorU;
use pathfinder_geometry::vector::vec2f;
use warp_core::ui::theme::color::internal_colors;
use warp_core::ui::theme::{Fill as ThemeFill, WarpTheme};
use warp_core::ui::{appearance::Appearance, Icon};
use warpui::elements::{
    ChildAnchor, ChildView, Dismiss, Empty, Hoverable, MainAxisSize, MouseStateHandle,
    ParentAnchor, ParentOffsetBounds, Rect, Shrinkable,
};
use warpui::platform::Cursor;
use warpui::ui_components::components::{UiComponent, UiComponentStyles};
use warpui::{
    elements::{
        Border, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, Fill, Flex,
        MainAxisAlignment, OffsetPositioning, Padding, ParentElement, Radius, Stack,
    },
    AppContext, Element, Entity, ModelHandle, SingletonEntity, View, WeakModelHandle,
};
use warpui::{TypedActionView, ViewContext, ViewHandle};

use warp_core::ui::theme::AnsiColorIdentifier;

#[cfg(feature = "local_fs")]
use crate::project_workspaces::PersistedWorkspaceEvent;
use crate::project_workspaces::{LSPEnablementResultForFile, LspRepoStatus, PersistedWorkspace};
use crate::ui_components::blended_colors;
#[cfg(feature = "local_fs")]
use crate::view_components::action_button::{ActionButton, ButtonSize, NakedTheme};
#[cfg(feature = "local_fs")]
use repo_metadata::repositories::DetectedRepositories;

const FOOTER_HEIGHT: f32 = 24.;
/// Margin around the LSP icon container
const ICON_MARGIN: f32 = 4.;
const INDICATOR_SIZE: f32 = 8.;

#[derive(Default)]
struct SingleFileMouseStates {
    open_logs: MouseStateHandle,
    restart_server: MouseStateHandle,
    stop_server: MouseStateHandle,
    start_server: MouseStateHandle,
    remove_server: MouseStateHandle,
}

struct FooterState {
    path: PathBuf,
    mouse_states: SingleFileMouseStates,
    lsp_repo_status: LspRepoStatus,
}

impl FooterState {
    fn path(&self) -> &Path {
        &self.path
    }

    fn cta_lsp_repo_status(&self) -> Option<&LspRepoStatus> {
        match &self.lsp_repo_status {
            status @ (LspRepoStatus::DisabledAndInstalled { .. }
            | LspRepoStatus::DisabledAndNotInstalled { .. }) => Some(status),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeFooterViewAction {
    CloseMenu,
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    ToggleMenu,
    EnableLSP,

    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    OpenLogs,
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    RestartServer,
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    StopServer,
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    StartServer,
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    RemoveServer,
}

enum LSPServerRenderStatus {
    Available,
    Stopped,
    Busy,
    Failed,
}

impl LSPServerRenderStatus {
    fn to_icon_color(&self, theme: &WarpTheme) -> ColorU {
        match self {
            LSPServerRenderStatus::Available => AnsiColorIdentifier::Green
                .to_ansi_color(&theme.terminal_colors().normal)
                .into(),
            LSPServerRenderStatus::Busy => AnsiColorIdentifier::Yellow
                .to_ansi_color(&theme.terminal_colors().normal)
                .into(),
            LSPServerRenderStatus::Failed => AnsiColorIdentifier::Red
                .to_ansi_color(&theme.terminal_colors().normal)
                .into(),
            LSPServerRenderStatus::Stopped => internal_colors::neutral_5(theme),
        }
    }

    fn render_status(server: Option<&LspServerModel>) -> Self {
        match server {
            Some(server) => match server.state() {
                LspModelState::Available { .. } if !server.has_pending_tasks() => {
                    LSPServerRenderStatus::Available
                }
                LspModelState::Starting | LspModelState::Available { .. } => {
                    LSPServerRenderStatus::Busy
                }
                LspModelState::Failed { .. } => LSPServerRenderStatus::Failed,
                LspModelState::Stopped { .. } | LspModelState::Stopping { .. } => {
                    LSPServerRenderStatus::Stopped
                }
            },
            None => LSPServerRenderStatus::Stopped,
        }
    }
}

pub struct CodeFooterView {
    state: FooterState,
    lsp_servers: Vec<WeakModelHandle<LspServerModel>>,
    lsp_status_button: ViewHandle<ActionButton>,
    enable_lsp_button: Option<ViewHandle<ActionButton>>,
    is_lsp_menu_open: bool,
}

impl CodeFooterView {
    fn create_lsp_status_button(
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<ActionButton> {
        ctx.add_typed_action_view(|ctx| {
            // Use Small size to get proper padding around the icon for a larger click target
            let mut button = ActionButton::new("", NakedTheme)
                .with_icon(Icon::Lightning)
                .with_size(ButtonSize::Small)
                .with_disabled_theme(NakedTheme)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(CodeFooterViewAction::ToggleMenu);
                });

            button.set_disabled(disabled, ctx);
            button
        })
    }
    pub fn new(path: PathBuf, ctx: &mut ViewContext<Self>) -> Self {
        let lsp_status_button = Self::create_lsp_status_button(true, ctx);

        let server_type = LanguageId::from_path(&path).map(|id| id.server_type());

        // Create a button that dispatches EnableLSP action
        // The action handler will check lsp_repo_status to decide whether to install first
        let enable_lsp_button = server_type.map(|st| {
            let label = format!("Enable {}", st.binary_name());
            ctx.add_typed_action_view(|_ctx| {
                ActionButton::new(label, NakedTheme)
                    .with_size(ButtonSize::Small)
                    .on_click(|ctx| {
                        ctx.dispatch_typed_action(CodeFooterViewAction::EnableLSP);
                    })
            })
        });

        // Kick off detection via PersistedWorkspace and subscribe for updates
        #[cfg(feature = "local_fs")]
        let initial_status = {
            let status = Self::detect_installation_status(&path, ctx);

            // Update button label based on initial status (handles cached results)
            if let Some(enable_button) = &enable_lsp_button {
                if let Some(label) = Self::button_label_for_status(&status) {
                    enable_button.update(ctx, |button, ctx| {
                        button.set_label(label, ctx);
                    });
                }
            }

            // Subscribe to InstallStatusUpdate events from PersistedWorkspace
            let persisted = PersistedWorkspace::handle(ctx);
            ctx.subscribe_to_model(&persisted, move |me, _model_handle, event, ctx| {
                // Only handle InstallStatusUpdate events for our server type
                let PersistedWorkspaceEvent::InstallStatusUpdate {
                    server_type: event_server_type,
                    status,
                } = event
                else {
                    return;
                };

                let path = &me.state.path;

                let Some(current_server_type) =
                    LanguageId::from_path(path).map(|id| id.server_type())
                else {
                    return;
                };

                // Only update if the event is for the server type we're tracking
                if *event_server_type != current_server_type {
                    return;
                }

                // Convert LSPInstallationStatus to LspRepoStatus
                let new_status =
                    LspRepoStatus::from_installation_status(status, *event_server_type);

                me.state.lsp_repo_status = new_status;
                me.update_enable_button_label(ctx);
                ctx.notify();
            });

            status
        };
        #[cfg(not(feature = "local_fs"))]
        let initial_status = LspRepoStatus::CheckingForInstallation;

        Self {
            state: FooterState {
                path,
                mouse_states: SingleFileMouseStates::default(),
                lsp_repo_status: initial_status,
            },
            lsp_servers: Vec::new(),
            is_lsp_menu_open: false,
            lsp_status_button,
            enable_lsp_button,
        }
    }

    /// Returns the appropriate button label for the given LSP repo status.
    fn button_label_for_status(status: &LspRepoStatus) -> Option<String> {
        match status {
            LspRepoStatus::DisabledAndNotInstalled { server_type } => {
                Some(format!("Install {}", server_type.binary_name()))
            }
            LspRepoStatus::DisabledAndInstalled { server_type } => {
                Some(format!("Enable {}", server_type.binary_name()))
            }
            _ => None,
        }
    }

    /// Detects LSP installation status for the given file path and returns the initial status.
    /// This is shared between `new` and `clear_server_subscription`.
    #[cfg(feature = "local_fs")]
    fn detect_installation_status(
        file_path: &std::path::Path,
        ctx: &mut ViewContext<Self>,
    ) -> LspRepoStatus {
        let server_type = LanguageId::from_path(file_path).map(|id| id.server_type());
        let Some(server_type) = server_type else {
            return LspRepoStatus::CheckingForInstallation;
        };

        let repo_root = DetectedRepositories::handle(ctx)
            .as_ref(ctx)
            .get_root_for_path(file_path)
            .or_else(|| file_path.parent().map(|p| p.to_path_buf()));

        let Some(repo_root) = repo_root else {
            return LspRepoStatus::CheckingForInstallation;
        };

        PersistedWorkspace::handle(ctx).update(ctx, |model, ctx| {
            model.detect_lsp_workspace_status(repo_root, server_type, ctx)
        })
    }

    /// Updates the enable button label based on the current CTA-worthy repo statuses.
    /// Hides the button when no CTAs remain.
    fn update_enable_button_label(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(label) = self
            .state
            .cta_lsp_repo_status()
            .and_then(Self::button_label_for_status)
        else {
            // No CTA-worthy statuses remain — hide the button.
            self.enable_lsp_button = None;
            return;
        };

        if let Some(enable_button) = &self.enable_lsp_button {
            enable_button.update(ctx, |button, ctx| {
                button.set_label(label, ctx);
            });
        } else {
            self.enable_lsp_button = Some(ctx.add_typed_action_view(|_ctx| {
                ActionButton::new(label, NakedTheme)
                    .with_size(ButtonSize::Small)
                    .on_click(|ctx| {
                        ctx.dispatch_typed_action(CodeFooterViewAction::EnableLSP);
                    })
            }));
        }
    }

    /// Subscribes to a single server's events and adds it.
    pub fn subscribe_to_server_events(
        &mut self,
        lsp_server: &ModelHandle<LspServerModel>,
        ctx: &mut ViewContext<Self>,
    ) {
        ctx.subscribe_to_model(lsp_server, |_, _, _, ctx| {
            ctx.notify();
        });

        self.lsp_servers = vec![lsp_server.downgrade()];
        self.state.lsp_repo_status = LspRepoStatus::Ready;

        self.lsp_status_button.update(ctx, |button, ctx| {
            button.set_disabled(false, ctx);
        });
        ctx.notify();
    }

    /// Clears the server subscription when a server is removed from the manager.
    /// This resets the footer to the "no server" state and kicks off installation detection.
    ///
    pub fn clear_server_subscription(&mut self, ctx: &mut ViewContext<Self>) {
        self.lsp_servers.clear();
        self.is_lsp_menu_open = false;

        self.lsp_status_button.update(ctx, |button, ctx| {
            button.set_disabled(true, ctx);
        });

        // Set initial status and kick off installation status detection
        #[cfg(feature = "local_fs")]
        {
            self.state.lsp_repo_status = Self::detect_installation_status(&self.state.path, ctx);
            self.update_enable_button_label(ctx);
        }
        #[cfg(not(feature = "local_fs"))]
        {
            self.state.lsp_repo_status = LspRepoStatus::CheckingForInstallation;
        }

        ctx.notify();
    }

    fn render_indicator(
        lsp_icon: Box<dyn Element>,
        color: ColorU,
        background_color: ColorU,
    ) -> Box<dyn Element> {
        let circle = Container::new(Empty::new().finish())
            .with_background(Fill::Solid(color))
            .with_corner_radius(CornerRadius::with_all(Radius::Percentage(50.)))
            .with_border(Border::all(1.5).with_border_fill(background_color))
            .finish();

        let inner_size = INDICATOR_SIZE;
        let inner_circle_constrained = ConstrainedBox::new(circle)
            .with_width(inner_size)
            .with_height(inner_size)
            .finish();

        let mut stack = Stack::new();
        stack.add_child(lsp_icon);

        let badge_offset = vec2f(-2., 2.);
        stack.add_positioned_child(
            inner_circle_constrained,
            OffsetPositioning::offset_from_parent(
                badge_offset,
                ParentOffsetBounds::Unbounded,
                ParentAnchor::TopRight,
                ChildAnchor::Center,
            ),
        );

        stack.finish()
    }

    fn render_menu_title(
        title: String,
        background: ThemeFill,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        appearance
            .ui_builder()
            .span(title)
            .with_style(UiComponentStyles {
                font_color: Some(appearance.theme().disabled_text_color(background).into()),
                font_size: Some(12.),
                ..Default::default()
            })
            .build()
            .with_vertical_padding(4.)
            .with_horizontal_padding(16.)
            .finish()
    }

    fn render_server_row(server: &LspServerModel, appearance: &Appearance) -> Box<dyn Element> {
        let render_status = LSPServerRenderStatus::render_status(Some(server));
        let failed_error = match server.state() {
            LspModelState::Failed { error } => Some(error.clone()),
            _ => None,
        };

        let background = appearance.theme().surface_2();
        let mut text_col = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(
                appearance
                    .ui_builder()
                    .span(server.server_name())
                    .build()
                    .finish(),
            );

        if let Some(error) = failed_error.filter(|e| !e.trim().is_empty()) {
            text_col.add_child(
                appearance
                    .ui_builder()
                    .span(error)
                    .with_style(UiComponentStyles {
                        font_color: Some(
                            appearance.theme().sub_text_color(background).into_solid(),
                        ),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            );
        }

        Flex::row()
            .with_child(
                Container::new(
                    ConstrainedBox::new(
                        Container::new(Empty::new().finish())
                            .with_background(Fill::Solid(
                                render_status.to_icon_color(appearance.theme()),
                            ))
                            .with_uniform_margin(2.)
                            .with_corner_radius(CornerRadius::with_all(Radius::Percentage(50.)))
                            .finish(),
                    )
                    .with_width(14.)
                    .with_height(14.)
                    .finish(),
                )
                .with_padding_right(8.)
                .finish(),
            )
            .with_child(Shrinkable::new(1., text_col.finish()).finish())
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_main_axis_size(MainAxisSize::Max)
            .finish()
    }

    fn render_menu_separator(appearance: &Appearance) -> Box<dyn Element> {
        Container::new(
            ConstrainedBox::new(
                Rect::new()
                    .with_background_color(blended_colors::neutral_4(appearance.theme()))
                    .finish(),
            )
            .with_height(1.)
            .finish(),
        )
        .with_vertical_padding(4.)
        .finish()
    }

    fn wrap_menu_in_dismiss(col: Flex, appearance: &Appearance) -> Box<dyn Element> {
        let background = appearance.theme().surface_2();
        Dismiss::new(
            ConstrainedBox::new(
                Container::new(col.finish())
                    .with_vertical_padding(8.)
                    .with_background(background)
                    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.)))
                    .with_border(
                        Border::all(1.)
                            .with_border_color(internal_colors::neutral_4(appearance.theme())),
                    )
                    .finish(),
            )
            .with_width(320.)
            .finish(),
        )
        .on_dismiss(|ctx, _app| ctx.dispatch_typed_action(CodeFooterViewAction::CloseMenu))
        .prevent_interaction_with_other_elements()
        .finish()
    }

    /// Renders the server details and actions for the open file.
    fn render_single_server_menu(
        &self,
        model: &LspServerModel,
        mouse_states: &SingleFileMouseStates,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let title = PersistedWorkspace::as_ref(app)
            .root_for_workspace(self.state.path())
            .and_then(|path| path.file_name())
            .and_then(|directory_name| directory_name.to_str().map(|s| s.to_string()))
            .unwrap_or("unknown workspace".to_string());

        let background = appearance.theme().surface_2();

        let mut col = Flex::column()
            .with_child(Self::render_menu_title(title, background, appearance))
            .with_child(
                Container::new(Self::render_server_row(model, appearance))
                    .with_vertical_padding(4.)
                    .with_horizontal_padding(16.)
                    .finish(),
            );

        // Add separator and action items based on server status
        let render_status = LSPServerRenderStatus::render_status(Some(model));
        let has_actions = matches!(
            render_status,
            LSPServerRenderStatus::Available
                | LSPServerRenderStatus::Stopped
                | LSPServerRenderStatus::Failed
        );
        if has_actions {
            col.add_child(Self::render_menu_separator(appearance));
        }

        if matches!(render_status, LSPServerRenderStatus::Available) {
            col.add_child(Self::render_open_logs_menu_item(mouse_states, appearance));
            col.add_child(Self::render_restart_server_menu_item(
                mouse_states,
                appearance,
            ));
            col.add_child(Self::render_stop_server_menu_item(mouse_states, appearance));
        }

        if matches!(render_status, LSPServerRenderStatus::Stopped) {
            col.add_child(Self::render_start_server_menu_item(
                mouse_states,
                appearance,
            ));
        }

        if matches!(render_status, LSPServerRenderStatus::Failed) {
            col.add_child(Self::render_open_logs_menu_item(mouse_states, appearance));
            col.add_child(Self::render_restart_server_menu_item(
                mouse_states,
                appearance,
            ));
            col.add_child(Self::render_remove_server_menu_item(
                mouse_states,
                appearance,
            ));
        }

        Self::wrap_menu_in_dismiss(col, appearance)
    }

    /// Generic rendering function for each action row in the menu. Note that we need to take in a closure here
    /// as Hoverable needs to dynamically construct its inner element.
    fn render_menu_item<F: 'static + Fn() -> Box<dyn Element>>(
        appearance: &Appearance,
        mouse_state: MouseStateHandle,
        icon_creator: F,
        label: &'static str,
        action: CodeFooterViewAction,
    ) -> Box<dyn Element> {
        let theme = appearance.theme().clone();
        let background = theme.surface_2();
        let ui_builder = appearance.ui_builder().clone();

        Hoverable::new(mouse_state, move |state| {
            let is_hovered = state.is_hovered();
            let text_color = theme.main_text_color(background).into();

            let icon_size = 14.;
            let icon = ConstrainedBox::new(icon_creator())
                .with_width(icon_size)
                .with_height(icon_size)
                .finish();

            let label_element = ui_builder
                .span(label)
                .with_style(UiComponentStyles {
                    font_color: Some(text_color),
                    ..Default::default()
                })
                .build()
                .finish();

            let row = Flex::row()
                .with_child(Container::new(icon).with_padding_right(8.).finish())
                .with_child(label_element)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_main_axis_size(MainAxisSize::Max)
                .finish();

            let hover_background = if is_hovered {
                Some(Fill::Solid(internal_colors::neutral_4(&theme)))
            } else {
                None
            };

            let container = Container::new(row)
                .with_horizontal_padding(16.)
                .with_vertical_padding(4.);

            let container = if let Some(bg) = hover_background {
                container.with_background(bg)
            } else {
                container
            };

            container.finish()
        })
        .with_cursor(Cursor::PointingHand)
        .on_click(move |ctx, _, _| {
            ctx.dispatch_typed_action(action.clone());
        })
        .finish()
    }
    fn render_open_logs_menu_item(
        mouse_states: &SingleFileMouseStates,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let background = theme.surface_2();
        let text_color: ColorU = theme.main_text_color(background).into();

        Self::render_menu_item(
            appearance,
            mouse_states.open_logs.clone(),
            move || {
                Icon::Code1
                    .to_warpui_icon(ThemeFill::Solid(text_color))
                    .finish()
            },
            "Open logs",
            CodeFooterViewAction::OpenLogs,
        )
    }
    fn render_restart_server_menu_item(
        mouse_states: &SingleFileMouseStates,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let background = theme.surface_2();
        let text_color: ColorU = theme.main_text_color(background).into();

        Self::render_menu_item(
            appearance,
            mouse_states.restart_server.clone(),
            move || {
                Icon::RefreshCcw
                    .to_warpui_icon(ThemeFill::Solid(text_color))
                    .finish()
            },
            "Restart server",
            CodeFooterViewAction::RestartServer,
        )
    }
    fn render_stop_server_menu_item(
        mouse_states: &SingleFileMouseStates,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let stop_color: ColorU = AnsiColorIdentifier::Red
            .to_ansi_color(&theme.terminal_colors().bright)
            .into();

        Self::render_menu_item(
            appearance,
            mouse_states.stop_server.clone(),
            move || {
                Container::new(Rect::new().with_background_color(stop_color).finish())
                    .with_uniform_padding(2.)
                    .finish()
            },
            "Stop server",
            CodeFooterViewAction::StopServer,
        )
    }
    fn render_start_server_menu_item(
        mouse_states: &SingleFileMouseStates,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let background = theme.surface_2();
        let text_color: ColorU = theme.main_text_color(background).into();

        Self::render_menu_item(
            appearance,
            mouse_states.start_server.clone(),
            move || {
                Icon::Play
                    .to_warpui_icon(ThemeFill::Solid(text_color))
                    .finish()
            },
            "Start server",
            CodeFooterViewAction::StartServer,
        )
    }
    fn render_remove_server_menu_item(
        mouse_states: &SingleFileMouseStates,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let background = theme.surface_2();
        let text_color: ColorU = theme.main_text_color(background).into();

        Self::render_menu_item(
            appearance,
            mouse_states.remove_server.clone(),
            move || {
                Icon::Trash
                    .to_warpui_icon(ThemeFill::Solid(text_color))
                    .finish()
            },
            "Remove server",
            CodeFooterViewAction::RemoveServer,
        )
    }
    /// Computes the aggregate indicator color across all tracked servers.
    /// Priority: Failed > Busy > Stopped > Available.
    fn aggregate_indicator_color(&self, theme: &WarpTheme, app: &AppContext) -> ColorU {
        if self.lsp_servers.is_empty() {
            return LSPServerRenderStatus::Stopped.to_icon_color(theme);
        }

        let mut worst = LSPServerRenderStatus::Available;
        for weak in &self.lsp_servers {
            let Some(server) = weak.upgrade(app) else {
                continue;
            };
            let status = LSPServerRenderStatus::render_status(Some(server.as_ref(app)));
            worst = match (&worst, &status) {
                (_, LSPServerRenderStatus::Failed) => LSPServerRenderStatus::Failed,
                (LSPServerRenderStatus::Failed, _) => LSPServerRenderStatus::Failed,
                (_, LSPServerRenderStatus::Busy) => LSPServerRenderStatus::Busy,
                (LSPServerRenderStatus::Busy, _) => LSPServerRenderStatus::Busy,
                (_, LSPServerRenderStatus::Stopped) => LSPServerRenderStatus::Stopped,
                (LSPServerRenderStatus::Stopped, _) => LSPServerRenderStatus::Stopped,
                _ => LSPServerRenderStatus::Available,
            };
        }
        worst.to_icon_color(theme)
    }

    /// Returns strong handles for all servers that can still be upgraded.
    fn live_servers(&self, app: &AppContext) -> Vec<ModelHandle<LspServerModel>> {
        self.lsp_servers
            .iter()
            .filter_map(|weak| weak.upgrade(app))
            .collect()
    }

    fn render_lsp_icon(&self, appearance: &Appearance, app: &AppContext) -> Box<dyn Element> {
        let theme = appearance.theme();
        let lsp_icon = ChildView::new(&self.lsp_status_button).finish();
        let background_color = theme.background().into_solid();

        let indicator_color = self.aggregate_indicator_color(theme, app);
        let indicator = Self::render_indicator(lsp_icon, indicator_color, background_color);

        let live = self.live_servers(app);
        if !self.is_lsp_menu_open || live.is_empty() {
            return indicator;
        }

        let server = live[0].as_ref(app);
        let menu =
            self.render_single_server_menu(server, &self.state.mouse_states, appearance, app);

        let mut element = Stack::new().with_child(indicator);
        element.add_positioned_child(
            menu,
            OffsetPositioning::offset_from_parent(
                vec2f(0., -2.),
                ParentOffsetBounds::WindowByPosition,
                ParentAnchor::TopLeft,
                ChildAnchor::BottomLeft,
            ),
        );
        element.finish()
    }

    fn render_status_text(
        theme: &WarpTheme,
        appearance: &Appearance,
        message: String,
    ) -> Box<dyn Element> {
        let status_content = appearance
            .ui_builder()
            .span(message)
            .with_style(UiComponentStyles {
                font_family_id: Some(appearance.ui_font_family()),
                font_color: Some(internal_colors::text_sub(theme, theme.background())),
                font_size: Some(12.0),
                ..Default::default()
            })
            .build()
            .finish();

        // Left margin only to separate from the icon; right margin removed to tighten
        // spacing between status text and the enable button action
        Container::new(status_content)
            .with_margin_left(ICON_MARGIN)
            .finish()
    }

    /// Returns a status message for a single server, if any.
    fn server_status_message(server: &LspServerModel) -> Option<String> {
        match server.state() {
            LspModelState::Available { .. } | LspModelState::Starting => server
                .latest_progress_update()
                .map(|update| update.to_display_message())
                .filter(|msg| !msg.trim().is_empty())
                .map(|msg| format!("{}: {msg}", server.server_name())),
            LspModelState::Stopped { .. } | LspModelState::Stopping { .. } => {
                Some(format!("{}: stopped", server.server_name()))
            }
            LspModelState::Failed { .. } => Some(format!("{}: error", server.server_name())),
        }
    }

    /// Computes the aggregated status message across all servers.
    /// Priority: failed error > starting/progress text > stopped text.
    fn compute_status_message(&self, app: &AppContext) -> (Option<String>, bool) {
        let live = self.live_servers(app);
        if !live.is_empty() {
            // Check for any failed server first
            for server in &live {
                let server_ref = server.as_ref(app);
                if let LspModelState::Failed { error } = server_ref.state() {
                    return (
                        Some(format!("{}: {error}", server_ref.server_name())),
                        false,
                    );
                }
            }
            // Then check for any starting/busy server
            for server in &live {
                let server_ref = server.as_ref(app);
                if let Some(msg) = Self::server_status_message(server_ref) {
                    if matches!(
                        server_ref.state(),
                        LspModelState::Starting | LspModelState::Available { .. }
                    ) {
                        return (Some(msg), false);
                    }
                }
            }
            // Then check stopped
            for server in &live {
                let server_ref = server.as_ref(app);
                if matches!(
                    server_ref.state(),
                    LspModelState::Stopped { .. } | LspModelState::Stopping { .. }
                ) {
                    return (
                        Some(format!("{}: stopped", server_ref.server_name())),
                        false,
                    );
                }
            }
            return (None, false);
        }

        let path = &self.state.path;
        let lsp_repo_status = &self.state.lsp_repo_status;
        match PersistedWorkspace::as_ref(app).has_enabled_lsp_server_for_file_path(path) {
            LSPEnablementResultForFile::UnsupportedLanguage => (
                Some("Language support is unavailable for this file type".to_string()),
                false,
            ),
            LSPEnablementResultForFile::LSPNotEnabled { root_name } => match lsp_repo_status {
                LspRepoStatus::CheckingForInstallation => (
                    Some(format!(
                        "Language support is not currently enabled for {}",
                        root_name.unwrap_or("this codebase".to_string())
                    )),
                    false,
                ),
                LspRepoStatus::Ready | LspRepoStatus::Enabled => (
                    Some("Language server is unavailable for this codebase".to_string()),
                    false,
                ),
                LspRepoStatus::DisabledAndNotInstalled { .. }
                | LspRepoStatus::DisabledAndInstalled { .. } => (
                    Some(format!(
                        "Language support is not currently enabled for {}",
                        root_name.unwrap_or("this codebase".to_string())
                    )),
                    true,
                ),
                LspRepoStatus::Installing { server_type } => (
                    Some(format!("Installing {}...", server_type.binary_name())),
                    false,
                ),
            },
            LSPEnablementResultForFile::Enabled => (None, false),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
pub enum CodeFooterViewEvent {
    EnableLSP { path: PathBuf },
    InstallAndEnableLSP { path: PathBuf },
    OpenLogs { path: PathBuf },
    RestartServer { server: ModelHandle<LspServerModel> },
    StopServer { server: ModelHandle<LspServerModel> },
    StartServer { server: ModelHandle<LspServerModel> },
}

impl Entity for CodeFooterView {
    type Event = CodeFooterViewEvent;
}

impl View for CodeFooterView {
    fn ui_name() -> &'static str {
        "CodeFooterView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();

        let mut footer_content = Flex::row()
            .with_main_axis_alignment(MainAxisAlignment::Start)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_main_axis_size(MainAxisSize::Max);

        footer_content.add_child(self.render_lsp_icon(appearance, app));

        let (status_message, should_show_enable_button) = self.compute_status_message(app);

        if let Some(status_message) = status_message {
            footer_content.add_child(
                Shrinkable::new(
                    1.,
                    Self::render_status_text(theme, appearance, status_message),
                )
                .finish(),
            );
        }

        if should_show_enable_button {
            if let Some(enable_lsp) = &self.enable_lsp_button {
                // Left margin only to separate from status text; right margin removed
                // to tighten padding between elements
                footer_content.add_child(
                    Container::new(ChildView::new(enable_lsp).finish())
                        .with_margin_left(ICON_MARGIN)
                        .finish(),
                );
            }
        }

        Container::new(
            ConstrainedBox::new(
                Container::new(footer_content.finish())
                    .with_padding(Padding::uniform(4.0))
                    .finish(),
            )
            .with_height(FOOTER_HEIGHT)
            .finish(),
        )
        .with_background_color(theme.background().into())
        .with_border(Border::top(2.0).with_border_fill(internal_colors::neutral_2(theme)))
        .with_corner_radius(CornerRadius::with_bottom(Radius::Pixels(8.)))
        .finish()
    }
}

impl TypedActionView for CodeFooterView {
    type Action = CodeFooterViewAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            CodeFooterViewAction::ToggleMenu => {
                self.is_lsp_menu_open = !self.is_lsp_menu_open;
                ctx.notify();
            }
            CodeFooterViewAction::CloseMenu => {
                self.is_lsp_menu_open = false;
                ctx.notify();
            }

            CodeFooterViewAction::EnableLSP => {
                let path = self.state.path().to_path_buf();
                match self.state.cta_lsp_repo_status() {
                    Some(LspRepoStatus::DisabledAndNotInstalled { .. }) => {
                        ctx.emit(CodeFooterViewEvent::InstallAndEnableLSP { path });
                    }
                    Some(LspRepoStatus::DisabledAndInstalled { .. }) => {
                        ctx.emit(CodeFooterViewEvent::EnableLSP { path });
                    }
                    _ => {}
                }
            }
            CodeFooterViewAction::OpenLogs => {
                self.is_lsp_menu_open = false;
                ctx.emit(CodeFooterViewEvent::OpenLogs {
                    path: self.state.path().to_path_buf(),
                });
                ctx.notify();
            }
            CodeFooterViewAction::RestartServer => {
                self.is_lsp_menu_open = false;
                if let Some(server) = self.lsp_servers.first().and_then(|w| w.upgrade(ctx)) {
                    ctx.emit(CodeFooterViewEvent::RestartServer {
                        server: server.clone(),
                    });
                }
                ctx.notify();
            }
            CodeFooterViewAction::StopServer => {
                self.is_lsp_menu_open = false;
                if let Some(server) = self.lsp_servers.first().and_then(|w| w.upgrade(ctx)) {
                    ctx.emit(CodeFooterViewEvent::StopServer {
                        server: server.clone(),
                    });
                }
                ctx.notify();
            }
            CodeFooterViewAction::StartServer => {
                self.is_lsp_menu_open = false;
                if let Some(server) = self.lsp_servers.first().and_then(|w| w.upgrade(ctx)) {
                    ctx.emit(CodeFooterViewEvent::StartServer {
                        server: server.clone(),
                    });
                }
                ctx.notify();
            }
            CodeFooterViewAction::RemoveServer => {
                self.is_lsp_menu_open = false;
                if let Some(server) = self.lsp_servers.first().and_then(|w| w.upgrade(ctx)) {
                    let workspace_root = server.as_ref(ctx).initial_workspace().to_path_buf();
                    let server_type = server.as_ref(ctx).server_type();

                    // Remove from manager (stops and removes)
                    LspManagerModel::handle(ctx).update(ctx, |manager, ctx| {
                        manager.remove_server(&workspace_root, server_type, ctx);
                    });

                    // Disable in PersistedWorkspace
                    PersistedWorkspace::handle(ctx).update(ctx, |workspace, _| {
                        workspace.disable_lsp_server_for_path(&workspace_root, server_type);
                    });
                }
                ctx.notify();
            }
        }
    }
}
