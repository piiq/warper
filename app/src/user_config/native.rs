use std::fs;
use std::path::Path;

use repo_metadata::RepositoryUpdate;
use warpui::{ModelContext, SingletonEntity};

use crate::features::FeatureFlag;
use crate::tab_configs::{TabConfig, TabConfigError};
use crate::themes::theme::WarpThemeConfig;
use crate::warp_managed_paths_watcher::{
    repository_update_touches_path, repository_update_touches_prefix, WarpManagedPathsWatcher,
    WarpManagedPathsWatcherEvent,
};

use super::util::{for_each_dir_entry, parse_single_theme_dir_entry, parse_tab_config_dir_entry};
use super::{tab_configs_dir, themes_dir, WarpConfigUpdateEvent};

impl super::WarpConfig {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        if FeatureFlag::TabConfigs.is_enabled() {
            let _ = ctx.spawn(
                async move { load_tab_configs(&tab_configs_dir()) },
                |me, (tab_configs, tab_config_errors), ctx| {
                    me.tab_configs = tab_configs;
                    me.tab_config_errors = tab_config_errors;
                    ctx.emit(WarpConfigUpdateEvent::TabConfigs);
                    // Don't emit TabConfigErrors on startup — the error toast
                    // should only appear when the user saves a config file,
                    // not on app restart.
                },
            );
        }
        ctx.subscribe_to_model(
            &WarpManagedPathsWatcher::handle(ctx),
            Self::handle_warp_managed_paths_event,
        );

        Self {
            theme_config: load_theme_configs(&themes_dir()),
            ..Default::default()
        }
    }

    fn handle_warp_managed_paths_event(
        &mut self,
        event: &WarpManagedPathsWatcherEvent,
        ctx: &mut ModelContext<Self>,
    ) {
        let WarpManagedPathsWatcherEvent::FilesChanged(update) = event;

        if update_touches_dir(update, &themes_dir()) {
            let theme_dir = themes_dir();
            let _ = ctx.spawn(
                async move { load_theme_configs(&theme_dir) },
                |me, theme_config, ctx| {
                    me.theme_config = theme_config;
                    ctx.emit(WarpConfigUpdateEvent::Themes);
                },
            );
        }

        if FeatureFlag::TabConfigs.is_enabled() && update_touches_dir(update, &tab_configs_dir()) {
            let tab_config_dir = tab_configs_dir();
            let _ = ctx.spawn(
                async move { load_tab_configs(&tab_config_dir) },
                |me, (configs, errors), ctx| {
                    me.tab_configs = configs;
                    me.tab_config_errors = errors.clone();
                    ctx.emit(WarpConfigUpdateEvent::TabConfigs);
                    if !errors.is_empty() {
                        ctx.emit(WarpConfigUpdateEvent::TabConfigErrors(errors));
                    }
                },
            );
        }

        if FeatureFlag::SettingsFile.is_enabled()
            && update_touches_path(update, &crate::settings::user_preferences_toml_file_path())
        {
            ctx.emit(WarpConfigUpdateEvent::Settings);
        }
    }
}

pub fn load_theme_configs(theme_path: &Path) -> WarpThemeConfig {
    let mut theme_configs = WarpThemeConfig::new();
    for_each_dir_entry(theme_path, parse_single_theme_dir_entry)
        .into_iter()
        .for_each(|(theme_name, theme)| theme_configs.add_new_theme(theme_name, theme));
    theme_configs
}

/// Loads all tab configs from `tab_config_path`. Each tab config is an individual TOML file.
///
/// Returns successfully parsed configs and any errors for files that failed to parse.
pub fn load_tab_configs(tab_config_path: &Path) -> (Vec<TabConfig>, Vec<TabConfigError>) {
    let results = for_each_dir_entry(tab_config_path, parse_tab_config_dir_entry);
    let mut configs = Vec::new();
    let mut errors = Vec::new();
    for result in results {
        match result {
            Ok(config) => configs.push(config),
            Err(error) => errors.push(error),
        }
    }
    configs.sort_by(|a, b| {
        let a_name = a.name.to_lowercase();
        let b_name = b.name.to_lowercase();
        a_name.cmp(&b_name).then_with(|| a.name.cmp(&b.name))
    });
    (configs, errors)
}

fn update_touches_dir(update: &RepositoryUpdate, path: &Path) -> bool {
    let canonical_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    repository_update_touches_prefix(update, path)
        || repository_update_touches_prefix(update, &canonical_path)
}

fn update_touches_path(update: &RepositoryUpdate, path: &Path) -> bool {
    let canonical_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    repository_update_touches_path(update, path)
        || repository_update_touches_path(update, &canonical_path)
}
