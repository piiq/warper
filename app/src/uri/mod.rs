mod docker;

use crate::root_view::open_new_window_get_handles;
use crate::workspace::Workspace;
use crate::{view_components::DismissibleToast, workspace::ToastStack};

use crate::settings_view::SettingsSection;
use crate::{quake_mode_window_id, quake_mode_window_is_open, safe_info, ChannelState, OpenPath};
use anyhow::{anyhow, ensure, Result};
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;
use warpui::notification::UserNotification;
use warpui::{platform::TerminationMode, SingletonEntity as _};

use warpui::{AppContext, WindowId};

use self::docker::open_docker_container;

const DESKTOP_REDIRECT_URI_PATH: &str = "/desktop_redirect";

#[derive(Debug, PartialEq, Eq)]
pub enum UriHost {
    /// A host prefix for all actions (e.g.: new tab, new window).
    Action,
    /// Supports opening warp's settings panel via URI
    Settings,
    /// A host prefix for a general-purpose home/landing page. Unlike other intent URIs, the home
    /// page behavior may change over time and vary from platform to platform.
    Home,
}

impl FromStr for UriHost {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "action" => Ok(Self::Action),
            "settings" => Ok(Self::Settings),
            "home" => Ok(Self::Home),
            _ => Err(anyhow!("Received url with unexpected host: {}", s)),
        }
    }
}

impl UriHost {
    fn handle(&self, primary_window_id: Option<WindowId>, url: &Url, ctx: &mut AppContext) {
        // Handle host
        match self {
            UriHost::Action => {
                match Action::parse(url) {
                    Ok(action) => action.handle(primary_window_id, url, ctx),
                    Err(err) => {
                        log::warn!("{err}");
                    }
                };
            }
            UriHost::Settings => {
                // We support opening different settings pages through URI:
                // - warp://settings/appearance - opens appearance settings page (themes, fonts, etc.)
                let settings_sub_page: Option<String> = url
                    .path_segments()
                    .into_iter()
                    .flatten()
                    .last()
                    .map(|s| s.to_string());
                if let Some(settings_sub_page) = settings_sub_page {
                    match settings_sub_page.as_str() {
                        "appearance" => {
                            dispatch_action_in_new_or_existing_window(
                                primary_window_id,
                                "root_view:open_settings_page_in_existing_window",
                                "root_view:open_settings_page_in_new_window",
                                &SettingsSection::Appearance,
                                ctx,
                            );
                        }
                        _ => {
                            log::warn!(
                                "Rejected unsupported local-only settings pane with uri={url}"
                            );
                        }
                    }
                } else {
                    log::warn!("Failed to open settings pane with uri={url}");
                }
            }
            UriHost::Home => {
                ctx.dispatch_global_action("root_view::open_new", &());
            }
        }
    }

    /// When handling this URI action, determine which window(s) should be focused.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn window_behavior_hint(&self) -> WindowBehaviorHint {
        use WindowBehaviorHint as W;
        match self {
            Self::Settings => W::default(),
            // These URLs always open new windows.
            Self::Home => W::Nothing,
            // This will actually be handled by [`Action::window_behavior_hint`].
            Self::Action => W::Nothing,
        }
    }
}

/// This determines which windows, if any, will become visible on handling a URI. This is a "hint"
/// because it is platform-dependent, and not all platforms can conform. For example, MacOS
/// automatically shows the frontmost window, and so the Nothing variant of this is impossible on
/// MacOS.
#[derive(Clone, Debug)]
enum WindowBehaviorHint {
    /// Determined by the [`get_primary_window`] function.
    ShowPrimaryWindow(WindowActivationFallbackBehavior),
    Nothing,
}

impl Default for WindowBehaviorHint {
    fn default() -> Self {
        Self::ShowPrimaryWindow(WindowActivationFallbackBehavior::NewWindow {
            replace_existing: false,
        })
    }
}

impl WindowBehaviorHint {
    /// Perform the desired window focus behavior for the URI being handled. This may change the
    /// "primary window" if a new one has to be created. Return the new primary WindowId.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn resolve(
        self,
        primary_window_id: Option<WindowId>,
        ctx: &mut AppContext,
    ) -> Option<WindowId> {
        match self {
            Self::ShowPrimaryWindow(fallback_behavior) => {
                if let Some(window_id) = primary_window_id {
                    match ctx.windows().windowing_system() {
                        Some(windowing_system)
                            if windowing_system.allows_programmatic_window_activation() =>
                        {
                            ctx.windows().show_window_and_focus_app(window_id);
                        }
                        _ => {
                            return fallback_behavior.resolve(window_id, ctx);
                        }
                    }
                }
            }
            Self::Nothing => {}
        };
        primary_window_id
    }
}

/// If we're in an environment where we can't fulfill [`WindowBehaviorHint`], and the OS default
/// behavior isn't acceptable/reliable, e.g. Wayland doesn't allow windows to programmatically show
/// themselves, try this fallback behavior instead.
#[derive(Clone, Debug)]
enum WindowActivationFallbackBehavior {
    /// If the primary window picked to handle the URL is not the active one, send a native push
    /// notification.
    Notify { title: String, description: String },
    /// Create a new window to handle the URI.
    NewWindow {
        /// Close the former "primary window" as determined by [`get_primary_window`]. This should
        /// generally default to `false` to avoid closing a window with information that the user
        /// may still want.
        replace_existing: bool,
    },
}

impl WindowActivationFallbackBehavior {
    /// Perform the desired window fallback behavior for the URI being handled. This may change the
    /// "primary window" if a new one has to be created. Return the new primary WindowId.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn resolve(self, primary_window_id: WindowId, ctx: &mut AppContext) -> Option<WindowId> {
        match self {
            WindowActivationFallbackBehavior::Notify { title, description } => {
                if ctx
                    .windows()
                    .active_window()
                    .is_some_and(|active_window| active_window == primary_window_id)
                {
                    return Some(primary_window_id);
                }
                if let Some(view_handle) = ctx
                    .views_of_type::<Workspace>(primary_window_id)
                    .filter(|views| !views.is_empty())
                    .map(|mut views| views.swap_remove(0))
                {
                    view_handle.update(ctx, |_, ctx| {
                        ctx.send_desktop_notification(
                            UserNotification::new(title, description, None),
                            |_, err, ctx| {
                                log::warn!(
                                    "Error showing URL intent notification on {:?}: {err:?}",
                                    ctx.window_id()
                                )
                            },
                        );
                    });
                }
                Some(primary_window_id)
            }
            WindowActivationFallbackBehavior::NewWindow { replace_existing } => {
                let new_window_id = open_new_window_get_handles(None, ctx).0;
                if replace_existing {
                    ctx.windows()
                        .close_window(primary_window_id, TerminationMode::Cancellable);
                }
                Some(new_window_id)
            }
        }
    }
}

/// Extract the `path` query parameter, expanding a leading `~` to the
/// user's home directory.
fn parse_tab_path(url: &Url) -> Option<PathBuf> {
    let raw = url.query_pairs().find(|(k, _)| k == "path")?.1;
    Some(PathBuf::from(shellexpand::tilde(&raw).into_owned()))
}

#[derive(Debug)]
enum Action {
    NewTab,
    NewWindow,
    Docker,
}

impl Action {
    fn parse(url: &Url) -> Result<Self> {
        match url.path() {
            "/new_tab" => Ok(Self::NewTab),
            "/new_window" => Ok(Self::NewWindow),
            "/docker/open_subshell" => Ok(Self::Docker),
            _ => Err(anyhow!(
                "Received \"action\" intent with unexpected action: {}",
                url.path()
            )),
        }
    }

    fn handle(&self, primary_window_id: Option<WindowId>, url: &Url, ctx: &mut AppContext) {
        #[cfg(target_os = "linux")]
        let primary_window_id = self.window_behavior_hint().resolve(primary_window_id, ctx);
        match self {
            Self::NewTab | Self::NewWindow => {
                let window_id = if let Self::NewTab = self {
                    primary_window_id
                } else {
                    None
                };
                let Some(path) = parse_tab_path(url) else {
                    log::warn!("Could not parse path to open a new tab/window");
                    return;
                };
                open_file(window_id, path, ctx);
            }
            Action::Docker => {
                if let Err(err) = open_docker_container(url, ctx) {
                    if let Some(window_id) = primary_window_id {
                        ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                            let toast =
                                DismissibleToast::error("Custom URI is invalid.".to_owned());
                            toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                        });
                    }

                    log::warn!("error opening docker container: {err}");
                }
            }
        }
    }

    /// When handling this URI action, determine which window(s) should be focused.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn window_behavior_hint(&self) -> WindowBehaviorHint {
        use WindowBehaviorHint as W;
        match self {
            Self::Docker => W::default(),
            Self::NewTab => W::ShowPrimaryWindow(WindowActivationFallbackBehavior::Notify {
                title: "New tab created".to_owned(),
                description: "Go to Warper to see your new tab.".to_owned(),
            }),
            Self::NewWindow => W::Nothing,
        }
    }
}

/// Handles all incoming file and local-only custom URLs.
pub fn handle_incoming_uri(url: &Url, ctx: &mut AppContext) {
    // Non-dogfood builds must never log the full URL here: URLs routed to this
    // handler can carry secrets in their query string. Log only the
    // non-sensitive components (scheme, host, path) on release channels;
    // dogfood builds retain the full URL for local debugging.
    safe_info!(
        safe: ("received url {}", safe_url_log_fields(url)),
        full: ("received url {:?}", &url)
    );

    // Pick the window that should be handling the URI.  This has some
    // additional logic to handle the hotkey window and there being no
    // currently-active window.
    let primary_window_id = get_primary_window(ctx.windows().frontmost_window_id(), ctx);

    // If we're running on a platform where we can spawn local TTYs,
    // check if this is a file:// URL and if so, spawn a new session
    // with an initial working directory based on the provided path.
    #[cfg(feature = "local_tty")]
    if url.scheme() == "file" {
        if let Ok(path) = url.to_file_path() {
            open_file(primary_window_id, path, ctx);
        }
        return;
    }

    match validate_custom_uri(url) {
        Ok(host) => {
            #[cfg(any(target_os = "linux", windows))]
            let primary_window_id = host.window_behavior_hint().resolve(primary_window_id, ctx);
            host.handle(primary_window_id, url, ctx);
        }
        Err(e) => {
            if let Some(window_id) = primary_window_id {
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = DismissibleToast::error(format!("Custom URI is invalid: {e:?}"));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
            }
            log::warn!("Custom URI is invalid: {e:?}");
        }
    }
}

/// Gets the primary window ID, and returns `None` if it does not exist.
/// A primary window is the foregrounded window, or one of the inactive non-quake windows.
/// A closed quake window is not counted.
fn get_primary_window(
    active_window_id: Option<WindowId>,
    ctx: &mut AppContext,
) -> Option<WindowId> {
    // Return quake mode window if it's open
    if let Some(window_id) = quake_mode_window_id()
        .filter(|window_id| quake_mode_window_is_open() && ctx.is_window_open(*window_id))
    {
        return Some(window_id);
    }

    // Otherwise, return active window
    if let Some(window_id) = active_window_id {
        return Some(window_id);
    }

    let mut non_quake_mode_windows = ctx
        .window_ids()
        .filter(|window_id| Some(*window_id) != quake_mode_window_id());

    // There's no active window, return first non-quake mode window or None if none exist.
    non_quake_mode_windows.next()
}

/// Open files in an external application and directories as terminal sessions.
fn open_file(window_id: Option<WindowId>, path: PathBuf, ctx: &mut AppContext) {
    if path.is_file() {
        #[cfg(feature = "local_fs")]
        if let Some(workspace) = window_id
            .and_then(|id| ctx.views_of_type::<Workspace>(id))
            .and_then(|views| views.into_iter().next())
        {
            let target = crate::util::openable_file_type::resolve_file_target(
                &path,
                crate::util::file::external_editor::EditorSettings::as_ref(ctx),
            );
            workspace.update(ctx, |workspace, ctx| {
                workspace.open_file_with_target(path, target, None, ctx)
            });
            return;
        }
        crate::util::file::open_file_path_in_external_editor(None, path, ctx);
        return;
    }
    if let Some((id, view)) = window_id.and_then(|id| ctx.root_view_id(id).map(|view| (id, view))) {
        ctx.dispatch_action(
            id,
            &[view],
            "root_view:add_session_at_path",
            &path,
            log::Level::Info,
        );
    } else {
        ctx.dispatch_global_action("root_view:open_new_from_path", &OpenPath { path });
    }
}

/// Helper function to dispatch an action to an existing window
/// or create new window if none exist.
fn dispatch_action_in_new_or_existing_window<T: 'static>(
    primary_window_id: Option<WindowId>,
    existing_window_action: &str,
    new_window_action: &str,
    args: &T,
    ctx: &mut AppContext,
) {
    let primary_window_and_view = primary_window_id.and_then(|window_id| {
        ctx.root_view_id(window_id)
            .map(|view_id| (window_id, view_id))
    });

    if let Some((primary_window_id, root_view_id)) = primary_window_and_view {
        ctx.dispatch_action(
            primary_window_id,
            &[root_view_id],
            existing_window_action,
            args,
            log::Level::Info,
        );
    } else {
        ctx.dispatch_global_action(new_window_action, args);
    }
}

/// Validates an incoming custom URI for security and returns the host.
fn validate_custom_uri(url: &Url) -> Result<UriHost> {
    // For now the only scheme we support is `[scheme_name]://[host_str]/...
    // Ignore all other urls that don't match this scheme for security purposes.
    if url.scheme() != ChannelState::url_scheme() {
        return Err(anyhow!(
            "Received url with unexpected scheme: {} ",
            url.scheme()
        ));
    }

    let host_str = url
        .host_str()
        .ok_or_else(|| anyhow!("Received url with no host str"))?;

    let host = UriHost::from_str(host_str)?;
    if matches!(host, UriHost::Settings) && !settings_uri_is_supported_local_page(url) {
        return Err(anyhow!(
            "Received unsupported local-only settings url: {}",
            url.path()
        ));
    }

    // Check if this host is allowed to have arbitrary paths.
    let host_allows_arbitrary_path = match host {
        UriHost::Action | UriHost::Settings => true,
        // Home only allows the desktop redirect path
        UriHost::Home => false,
    };

    ensure!(
        host_allows_arbitrary_path || url.path() == DESKTOP_REDIRECT_URI_PATH,
        "Received url with unexpected path: {} ",
        url.path()
    );

    Ok(host)
}

fn settings_uri_is_supported_local_page(url: &Url) -> bool {
    let Some(settings_sub_page) = url.path_segments().into_iter().flatten().last() else {
        return false;
    };

    settings_sub_page == "appearance"
}

/// Formats the non-sensitive components of an incoming URL for logging on
/// release channels.
///
/// The returned string contains only the URL's scheme, host, and path — never
/// its query string, fragment, or userinfo component. URLs that reach
/// [`handle_incoming_uri`] can carry secrets in their query, so this helper
/// exists to give [`safe_info!`] a redacted representation that
/// still preserves enough signal for triage.
///
/// `url.host_str()` can return `None` for schemes that don't require a host
/// (e.g. some `file://` URLs on certain platforms); the literal `-` is used
/// as a placeholder in that case so the formatter never panics.
fn safe_url_log_fields(url: &Url) -> String {
    format!(
        "scheme={} host={} path={}",
        url.scheme(),
        url.host_str().unwrap_or("-"),
        url.path(),
    )
}

#[cfg(test)]
#[path = "uri_test.rs"]
mod tests;
