use super::*;
use crate::app_state::{get_app_state, TabSnapshot};
use crate::menu::MenuAction;
use crate::pane_group::pane::PaneEvent;
use crate::pane_group::{PaneContent, SettingsPane};
use crate::root_view;
use crate::settings_view::SettingsViewEvent;
use crate::view_components::ToastFlavor;
use crate::workspace::VerticalTabsPaneContextMenuTarget;
use warpui::platform::app::{AppCallbackDispatcher, AppCallbacks};
use warpui::WindowId;

fn initialize_transfer_test(app: &mut App) {
    initialize_app(app);
    app.update(root_view::init);
}

fn add_settings_pane(group: &ViewHandle<PaneGroup>, app: &mut App) -> PaneId {
    group.update(app, |group, ctx| {
        let pane = SettingsPane::new(SettingsSection::Appearance, None, ctx.window_id(), ctx);
        let id = pane.id();
        group.add_pane_with_direction(Direction::Right, pane, true, ctx);
        id
    })
}

#[test]
fn moved_tab_discards_closed_settings_without_moving_source_settings_view() {
    let _undo_guard = FeatureFlag::UndoClosedPanes.override_enabled(true);
    for reopen_in_source in [false, true] {
        App::test((), move |mut app| async move {
            initialize_transfer_test(&mut app);
            let source = three_tab_workspace(&mut app);
            let source_window = source.read(&app, |workspace, _| workspace.window_id);
            let group = source.read(&app, |workspace, _| workspace.tabs[0].pane_group.clone());
            let closed_settings = add_settings_pane(&group, &mut app);
            group.update(&mut app, |group, ctx| {
                group.close_pane(closed_settings, ctx)
            });
            let closed_terminal = group.update(&mut app, |group, ctx| {
                group.handle_action(&PaneGroupAction::Add(Direction::Down), ctx);
                let pane = group.focused_pane_id(ctx);
                group.close_pane(pane, ctx);
                pane
            });
            let settings_view = source.update(&mut app, |workspace, ctx| {
                if reopen_in_source {
                    workspace.open_settings_pane(None, None, ctx);
                }
                workspace.settings_pane.clone()
            });
            let source_locator =
                app.read(|ctx| SettingsPaneManager::as_ref(ctx).find_pane(source_window));

            app.update(|ctx| ctx.windows().fail_next_open_window_for_test());
            source.update(&mut app, |workspace, ctx| {
                workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(0), ctx);
            });
            group.read(&app, |group, _| assert!(group.has_pane_id(closed_settings)));
            assert_eq!(app.window_ids().len(), 1);

            source.update(&mut app, |workspace, ctx| {
                workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(0), ctx);
            });

            let destination = destination_workspace(&mut app, source_window);
            let destination_window = destination.read(&app, |workspace, _| workspace.window_id);
            group.read(&app, |group, _| {
                assert!(!group.has_pane_id(closed_settings));
                assert!(group.has_pane_id(closed_terminal));
                assert!(group.is_pane_hidden_for_close(closed_terminal));
                assert_eq!(group.visible_pane_count(), 1);
            });
            SettingsPaneManager::handle(&app).read(&app, |manager, ctx| {
                assert_eq!(settings_view.window_id(ctx), source_window);
                assert_eq!(
                    manager.settings_view(source_window).id(),
                    settings_view.id()
                );
                assert_eq!(manager.find_pane(source_window), source_locator);
                assert_eq!(manager.find_pane(destination_window), None);
                assert_eq!(
                    manager.settings_view(destination_window).window_id(ctx),
                    destination_window
                );
            });
            destination.update(&mut app, |workspace, ctx| {
                workspace.open_settings_pane(None, None, ctx);
                assert_eq!(workspace.tab_count(), 2);
                assert_eq!(workspace.settings_pane.window_id(ctx), destination_window);
            });
        });
    }
}

#[test]
fn moved_tab_keeps_reopened_settings_pane_events_after_closed_wrapper_cleanup() {
    let _undo_guard = FeatureFlag::UndoClosedPanes.override_enabled(true);
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = three_tab_workspace(&mut app);
        let source_window = source.read(&app, |workspace, _| workspace.window_id);
        let group = source.read(&app, |workspace, _| workspace.tabs[0].pane_group.clone());
        let closed_settings = add_settings_pane(&group, &mut app);
        group.update(&mut app, |group, ctx| {
            group.close_pane(closed_settings, ctx)
        });
        let live_settings = add_settings_pane(&group, &mut app);
        let settings_view = source.update(&mut app, |workspace, ctx| {
            let view = workspace.settings_pane.clone();
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(0), ctx);
            view
        });
        let destination = destination_workspace(&mut app, source_window);
        destination.read(&app, |workspace, ctx| {
            assert_eq!(workspace.settings_pane.id(), settings_view.id());
            assert_eq!(
                SettingsPaneManager::as_ref(ctx).find_pane(workspace.window_id),
                Some(PaneViewLocator {
                    pane_group_id: group.id(),
                    pane_id: live_settings,
                })
            );
        });
        group.update(&mut app, |group, ctx| {
            assert!(!group.has_pane_id(closed_settings));
            assert!(group.has_pane_id(live_settings));
            let terminal = group.pane_id_by_index(0).unwrap();
            group.focus_pane_by_id(terminal, ctx);
        });

        settings_view.update(&mut app, |_, ctx| {
            ctx.emit(SettingsViewEvent::Pane(PaneEvent::FocusSelf));
        });

        group.read(&app, |group, ctx| {
            assert_eq!(group.focused_pane_id(ctx), live_settings);
        });
    });
}

#[test]
fn moved_tab_keeps_source_settings_undo_in_its_original_window() {
    let _undo_guard = FeatureFlag::UndoClosedPanes.override_enabled(true);
    for (close_whole_tab, close_destination) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        App::test((), move |mut app| async move {
            initialize_transfer_test(&mut app);
            let source = three_tab_workspace(&mut app);
            let source_window = source.read(&app, |workspace, _| workspace.window_id);
            let (closed_group, moved_group) = source.read(&app, |workspace, _| {
                (
                    workspace.tabs[0].pane_group.clone(),
                    workspace.tabs[1].pane_group.clone(),
                )
            });
            let closed_settings = add_settings_pane(&closed_group, &mut app);
            if close_whole_tab {
                source.update(&mut app, |workspace, ctx| {
                    workspace.close_tab(0, true, true, ctx);
                });
            } else {
                closed_group.update(&mut app, |group, ctx| {
                    group.close_pane(closed_settings, ctx);
                });
            }
            add_settings_pane(&moved_group, &mut app);
            source.update(&mut app, |workspace, ctx| {
                let index = workspace
                    .tabs
                    .iter()
                    .position(|tab| tab.pane_group.id() == moved_group.id())
                    .unwrap();
                workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(index), ctx);
            });
            let destination = destination_workspace(&mut app, source_window);
            let moved_settings =
                destination.read(&app, |workspace, _| workspace.settings_pane.clone());
            let source_settings = source.read(&app, |workspace, _| workspace.settings_pane.clone());
            let destination_window = destination.read(&app, |workspace, _| workspace.window_id);
            if close_destination {
                let mut dispatcher = AppCallbackDispatcher::new(
                    AppCallbacks {
                        on_window_will_close: Some(Box::new(|data, _| assert!(data.is_some()))),
                        ..Default::default()
                    },
                    app.clone(),
                );
                dispatcher.window_will_close(destination_window);
            }

            UndoCloseStack::handle(&app).update(&mut app, |stack, ctx| stack.undo_close(ctx));

            closed_group.read(&app, |group, ctx| {
                assert!(!group.is_pane_hidden_for_close(closed_settings));
                let roots = group
                    .downcast_pane_by_id::<SettingsPane>(closed_settings)
                    .unwrap()
                    .window_transfer_roots(ctx);
                assert!(roots.contains(&source_settings.id()));
                assert!(!roots.contains(&moved_settings.id()));
                assert_eq!(
                    SettingsPaneManager::as_ref(ctx).find_pane(source_window),
                    Some(PaneViewLocator {
                        pane_group_id: closed_group.id(),
                        pane_id: closed_settings,
                    })
                );
            });
            app.update(|ctx| assert!(source_settings.is_self_or_child_focused(ctx)));

            closed_group.update(&mut app, |group, ctx| {
                let terminal = group.pane_id_by_index(0).unwrap();
                group.focus_pane_by_id(terminal, ctx);
            });
            source_settings.update(&mut app, |_, ctx| {
                ctx.emit(SettingsViewEvent::Pane(PaneEvent::FocusSelf));
            });
            closed_group.read(&app, |group, ctx| {
                assert_eq!(group.focused_pane_id(ctx), closed_settings);
            });
            app.read(|ctx| {
                assert_eq!(ctx.is_window_open(destination_window), !close_destination);
                if !close_destination {
                    assert_eq!(moved_settings.window_id(ctx), destination_window);
                }
            });
        });
    }
}

#[test]
fn moved_tab_forgets_discarded_settings_undo_and_preserves_terminal_and_tab_undo() {
    let _undo_guard = FeatureFlag::UndoClosedPanes.override_enabled(true);
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = three_tab_workspace(&mut app);
        let closed_tab = source.update(&mut app, |workspace, ctx| {
            let tab = workspace.tabs[2].pane_group.id();
            workspace.close_tab(2, true, true, ctx);
            tab
        });
        let group = source.read(&app, |workspace, _| workspace.tabs[0].pane_group.clone());
        let closed_terminal = group.update(&mut app, |group, ctx| {
            group.handle_action(&PaneGroupAction::Add(Direction::Down), ctx);
            group.focused_pane_id(ctx)
        });
        let closed_settings = add_settings_pane(&group, &mut app);
        group.update(&mut app, |group, ctx| {
            group.close_pane(closed_terminal, ctx);
            group.close_pane(closed_settings, ctx);
            assert!(group.is_pane_hidden_for_close(closed_terminal));
            assert!(group.is_pane_hidden_for_close(closed_settings));
        });
        source.update(&mut app, |workspace, ctx| {
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(0), ctx);
        });

        UndoCloseStack::handle(&app).update(&mut app, |stack, ctx| stack.undo_close(ctx));

        group.read(&app, |group, _| {
            assert!(!group.has_pane_id(closed_settings));
            assert!(!group.is_pane_hidden_for_close(closed_terminal));
            assert_eq!(group.visible_pane_count(), 2);
        });

        UndoCloseStack::handle(&app).update(&mut app, |stack, ctx| stack.undo_close(ctx));

        source.read(&app, |workspace, _| {
            assert_eq!(workspace.tab_count(), 2);
            assert!(workspace.tab_views().any(|group| group.id() == closed_tab));
        });
        assert!(UndoCloseStack::handle(&app).read(&app, |stack, _| stack.is_empty()));
    });
}

fn three_tab_workspace(app: &mut App) -> ViewHandle<Workspace> {
    let workspace = mock_workspace(app);
    workspace.update(app, |workspace, ctx| {
        workspace.add_terminal_tab(ctx);
        workspace.add_terminal_tab(ctx);
    });
    workspace
}

fn destination_workspace(app: &mut App, source_window_id: WindowId) -> ViewHandle<Workspace> {
    let destination_ids = app
        .window_ids()
        .into_iter()
        .filter(|id| *id != source_window_id)
        .collect::<Vec<_>>();
    assert_eq!(destination_ids.len(), 1);
    app.update(|ctx| root_view::workspace_for_window(destination_ids[0], ctx).unwrap())
}

fn tab_ids(workspace: &Workspace) -> Vec<EntityId> {
    workspace
        .tabs
        .iter()
        .map(|tab| tab.pane_group.id())
        .collect()
}

fn tab_snapshot(workspace: &ViewHandle<Workspace>, index: usize, app: &App) -> TabSnapshot {
    workspace.read(app, |workspace, ctx| {
        workspace.snapshot(workspace.window_id, false, ctx).tabs[index].clone()
    })
}

fn move_menu_item_index(workspace: &Workspace, ctx: &AppContext) -> Option<usize> {
    workspace.tab_right_click_menu.read(ctx, |menu, _| {
        menu.items().iter().position(|item| {
            matches!(item, MenuItem::Item(fields) if fields.label() == "Move Tab to New Window")
        })
    })
}

fn select_move_menu_item(workspace: &ViewHandle<Workspace>, app: &mut App) {
    let menu = workspace.update(app, |workspace, ctx| {
        let item_index = move_menu_item_index(workspace, ctx).expect("move item is available");
        workspace.tab_right_click_menu.update(ctx, |menu, ctx| {
            menu.set_selected_by_index(item_index, ctx);
        });
        workspace.tab_right_click_menu.clone()
    });
    menu.update(app, |menu, ctx| menu.handle_action(&MenuAction::Enter, ctx));
}

#[test]
fn move_tab_to_new_window_preserves_inactive_tab_and_source_selection() {
    let _vertical_tabs_guard = FeatureFlag::VerticalTabs.override_enabled(true);
    for vertical_tabs in [false, true] {
        App::test((), move |mut app| async move {
            initialize_transfer_test(&mut app);
            TabSettings::handle(&app).update(&mut app, |settings, ctx| {
                settings
                    .use_vertical_tabs
                    .set_value(vertical_tabs, ctx)
                    .unwrap();
            });
            let source = three_tab_workspace(&mut app);
            let source_window_id = source.read(&app, |workspace, _| workspace.window_id);
            let original_ids = source.update(&mut app, |workspace, ctx| {
                workspace.tabs[0].selected_color =
                    SelectedTabColor::Color(AnsiColorIdentifier::Blue);
                workspace.tabs[0].default_directory_color = Some(AnsiColorIdentifier::Green);
                workspace.tabs[0].pane_group.update(ctx, |group, ctx| {
                    group.set_title("build logs", ctx);
                });
                workspace.activate_tab(2, ctx);
                let ids = tab_ids(workspace);
                workspace.toggle_tab_right_click_menu(
                    0,
                    TabContextMenuAnchor::Pointer(Vector2F::zero()),
                    ctx,
                );
                ids
            });
            let before = tab_snapshot(&source, 0, &app);

            select_move_menu_item(&source, &mut app);

            let destination = destination_workspace(&mut app, source_window_id);
            source.read(&app, |workspace, _| {
                assert_eq!(tab_ids(workspace), original_ids[1..]);
                assert_eq!(workspace.active_tab_index(), 1);
                assert!(workspace.show_tab_right_click_menu.is_none());
            });
            destination.read(&app, |workspace, ctx| {
                assert_eq!(tab_ids(workspace), vec![original_ids[0]]);
                assert_eq!(workspace.get_tab_color(0), Some(AnsiColorIdentifier::Blue));
                assert_eq!(
                    workspace.tabs[0].pane_group.window_id(ctx),
                    workspace.window_id
                );
            });
            assert_eq!(tab_snapshot(&destination, 0, &app), before);
            assert!(UndoCloseStack::handle(&app).read(&app, |stack, _| stack.is_empty()));

            let state = app.read(get_app_state);
            assert_eq!(state.windows.len(), 2);
            assert_eq!(
                state
                    .windows
                    .iter()
                    .map(|window| window.tabs.len())
                    .sum::<usize>(),
                3
            );
            assert_eq!(
                state
                    .windows
                    .iter()
                    .flat_map(|window| &window.tabs)
                    .filter(|tab| { tab.custom_title.as_deref() == Some("build logs") })
                    .count(),
                1,
            );
        });
    }
}

#[test]
fn move_tab_to_new_window_selects_preceding_tab_or_first_remaining_tab() {
    for (moved_index, expected_active_index) in [(0, 0), (1, 0), (2, 1)] {
        App::test((), move |mut app| async move {
            initialize_transfer_test(&mut app);
            let source = three_tab_workspace(&mut app);
            let original_ids = source.update(&mut app, |workspace, ctx| {
                workspace.activate_tab(moved_index, ctx);
                let ids = tab_ids(workspace);
                workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(moved_index), ctx);
                ids
            });

            source.read(&app, |workspace, _| {
                let mut expected_ids = original_ids.clone();
                expected_ids.remove(moved_index);
                assert_eq!(tab_ids(workspace), expected_ids);
                assert_eq!(workspace.active_tab_index(), expected_active_index);
            });
        });
    }
}

#[test]
fn move_tab_to_new_window_menu_is_absent_for_final_tab_and_dismissal_keeps_tab() {
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = mock_workspace(&mut app);
        let original_id = source.update(&mut app, |workspace, ctx| {
            workspace.toggle_tab_right_click_menu(
                0,
                TabContextMenuAnchor::Pointer(Vector2F::zero()),
                ctx,
            );
            assert!(move_menu_item_index(workspace, ctx).is_none());
            workspace.handle_tab_right_click_menu_event(
                &MenuEvent::Close {
                    via_select_item: false,
                },
                ctx,
            );
            workspace.add_terminal_tab(ctx);
            workspace.toggle_tab_right_click_menu(
                0,
                TabContextMenuAnchor::Pointer(Vector2F::zero()),
                ctx,
            );
            assert!(move_menu_item_index(workspace, ctx).is_some());
            let id = workspace.tabs[0].pane_group.id();
            workspace.handle_tab_right_click_menu_event(
                &MenuEvent::Close {
                    via_select_item: false,
                },
                ctx,
            );
            id
        });
        assert_eq!(app.window_ids().len(), 1);
        source.read(&app, |workspace, _| {
            assert_eq!(workspace.tab_count(), 2);
            assert_eq!(workspace.tabs[0].pane_group.id(), original_id);
            assert!(workspace.show_tab_right_click_menu.is_none());
        });
    });
}

#[test]
fn move_tab_to_new_window_from_vertical_pane_menu_preserves_split_focus_and_input() {
    let _vertical_tabs_guard = FeatureFlag::VerticalTabs.override_enabled(true);
    for panel_open in [false, true] {
        App::test((), move |mut app| async move {
            initialize_transfer_test(&mut app);
            TabSettings::handle(&app).update(&mut app, |settings, ctx| {
                settings.use_vertical_tabs.set_value(true, ctx).unwrap();
            });
            let source = three_tab_workspace(&mut app);
            let source_window_id = source.read(&app, |workspace, _| workspace.window_id);
            let (group, focused_pane, input, terminals) =
                source.update(&mut app, |workspace, ctx| {
                    workspace.activate_tab(1, ctx);
                    workspace.vertical_tabs_panel_open = panel_open;
                    let group = workspace.active_tab_pane_group().clone();
                    group.update(ctx, |group, ctx| {
                        group.handle_action(&PaneGroupAction::Add(Direction::Right), ctx);
                        group.handle_action(&PaneGroupAction::Add(Direction::Down), ctx);
                    });
                    let focused_pane = group.read(ctx, |group, ctx| group.focused_pane_id(ctx));
                    workspace.set_custom_pane_name(
                        PaneViewLocator {
                            pane_group_id: group.id(),
                            pane_id: focused_pane,
                        },
                        "tests".to_string(),
                        ctx,
                    );
                    workspace
                        .set_active_terminal_input_contents_and_focus_app("echo retained", ctx);
                    let input = workspace.get_active_input_view_handle(ctx).unwrap();
                    group.update(ctx, |group, ctx| {
                        group.handle_action(&PaneGroupAction::ToggleMaximizePane, ctx);
                    });
                    workspace.toggle_vertical_tabs_pane_context_menu(
                        1,
                        VerticalTabsPaneContextMenuTarget::ClickedPane(PaneViewLocator {
                            pane_group_id: group.id(),
                            pane_id: group.read(ctx, |group, _| group.pane_id_by_index(0).unwrap()),
                        }),
                        Vector2F::zero(),
                        ctx,
                    );
                    let terminals = group.read(ctx, |group, ctx| {
                        (0..group.pane_count())
                            .map(|index| {
                                group
                                    .terminal_view_from_pane_id(
                                        group.pane_id_by_index(index).unwrap(),
                                        ctx,
                                    )
                                    .unwrap()
                            })
                            .collect::<Vec<_>>()
                    });
                    (group, focused_pane, input, terminals)
                });
            let before = tab_snapshot(&source, 1, &app);

            select_move_menu_item(&source, &mut app);

            let destination = destination_workspace(&mut app, source_window_id);
            assert_eq!(tab_snapshot(&destination, 0, &app), before);
            destination.read(&app, |workspace, ctx| {
                assert_eq!(workspace.tab_count(), 1);
                assert_eq!(workspace.active_tab_pane_group().id(), group.id());
                assert_eq!(workspace.vertical_tabs_panel_open, panel_open);
                group.read(ctx, |group, ctx| {
                    assert_eq!(group.pane_count(), 3);
                    assert_eq!(group.focused_pane_id(ctx), focused_pane);
                    assert!(group
                        .focus_state_handle()
                        .as_ref(ctx)
                        .is_focused_pane_maximized());
                });
                for terminal in &terminals {
                    assert_eq!(terminal.window_id(ctx), workspace.window_id);
                    let terminal_input = terminal.as_ref(ctx).input();
                    assert_eq!(terminal_input.window_id(ctx), workspace.window_id);
                    assert_eq!(
                        terminal_input.as_ref(ctx).editor().window_id(ctx),
                        workspace.window_id
                    );
                }
                assert_eq!(input.window_id(ctx), workspace.window_id);
                assert_eq!(input.as_ref(ctx).buffer_text(ctx), "echo retained");
                assert!(input.as_ref(ctx).editor().is_focused(ctx));
            });
        });
    }
}

#[test]
fn move_tab_to_new_window_rejects_stale_tab_without_changing_source() {
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = three_tab_workspace(&mut app);
        let original_ids = source.update(&mut app, |workspace, ctx| {
            let ids = tab_ids(workspace);
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(usize::MAX), ctx);
            ids
        });
        assert_eq!(app.window_ids().len(), 1);
        source.read(&app, |workspace, ctx| {
            assert_eq!(tab_ids(workspace), original_ids);
            assert!(workspace.toast_stack.as_ref(ctx).has_toasts());
        });
    });
}

#[test]
fn moved_tab_events_are_handled_by_destination_workspace() {
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = three_tab_workspace(&mut app);
        let source_window_id = source.read(&app, |workspace, _| workspace.window_id);
        let group = source.update(&mut app, |workspace, ctx| {
            let group = workspace.tabs[0].pane_group.clone();
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(0), ctx);
            group
        });
        let destination = destination_workspace(&mut app, source_window_id);

        for workspace in [&source, &destination] {
            workspace.update(&mut app, |workspace, ctx| {
                workspace
                    .toast_stack
                    .update(ctx, |toasts, ctx| toasts.clear_toasts(ctx));
            });
        }
        group.update(&mut app, |_, ctx| {
            ctx.emit(pane_group::Event::ShowToast {
                message: "Command completed".to_string(),
                flavor: ToastFlavor::Success,
                pane_id: None,
            });
        });

        source.read(&app, |workspace, ctx| {
            assert!(!workspace.toast_stack.as_ref(ctx).has_toasts());
        });
        destination.read(&app, |workspace, ctx| {
            assert!(workspace.toast_stack.as_ref(ctx).has_toasts());
        });
    });
}

#[test]
fn move_tab_to_new_window_keeps_source_usable_when_window_creation_fails() {
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = three_tab_workspace(&mut app);
        let (original_ids, input) = source.update(&mut app, |workspace, ctx| {
            workspace.set_active_terminal_input_contents_and_focus_app("echo source", ctx);
            (
                tab_ids(workspace),
                workspace.get_active_input_view_handle(ctx).unwrap(),
            )
        });
        let before = tab_snapshot(&source, 2, &app);
        app.update(|ctx| ctx.windows().fail_next_open_window_for_test());

        source.update(&mut app, |workspace, ctx| {
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(2), ctx);
        });

        assert_eq!(app.window_ids().len(), 1);
        assert_eq!(tab_snapshot(&source, 2, &app), before);
        source.read(&app, |workspace, ctx| {
            assert_eq!(tab_ids(workspace), original_ids);
            assert_eq!(workspace.active_tab_index(), 2);
            assert!(workspace.toast_stack.as_ref(ctx).has_toasts());
            assert_eq!(input.window_id(ctx), workspace.window_id);
            assert_eq!(input.as_ref(ctx).buffer_text(ctx), "echo source");
            assert!(input.as_ref(ctx).editor().is_focused(ctx));
        });
        assert!(UndoCloseStack::handle(&app).read(&app, |stack, _| stack.is_empty()));
    });
}

#[test]
fn move_tab_to_new_window_reassigns_settings_pane_and_its_events() {
    App::test((), |mut app| async move {
        initialize_transfer_test(&mut app);
        let source = mock_workspace(&mut app);
        let source_window_id = source.read(&app, |workspace, _| workspace.window_id);
        let (settings_view, locator) = source.update(&mut app, |workspace, ctx| {
            workspace.open_settings_pane(None, None, ctx);
            let locator = SettingsPaneManager::as_ref(ctx)
                .find_pane(source_window_id)
                .unwrap();
            let settings_view = workspace.settings_pane.clone();
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(1), ctx);
            (settings_view, locator)
        });
        let destination = destination_workspace(&mut app, source_window_id);
        let destination_window_id = destination.read(&app, |workspace, _| workspace.window_id);

        SettingsPaneManager::handle(&app).read(&app, |manager, ctx| {
            let source_settings = manager.settings_view(source_window_id);
            assert_eq!(source_settings.window_id(ctx), source_window_id);
            assert_ne!(source_settings.id(), settings_view.id());
            assert_eq!(manager.find_pane(source_window_id), None);
            assert_eq!(manager.find_pane(destination_window_id), Some(locator));
            assert_eq!(
                manager.settings_view(destination_window_id).id(),
                settings_view.id()
            );
            assert_eq!(settings_view.window_id(ctx), destination_window_id);
        });

        destination.update(&mut app, |workspace, ctx| {
            assert_eq!(workspace.settings_pane.id(), settings_view.id());
            workspace.open_settings_pane(None, None, ctx);
            assert_eq!(workspace.tab_count(), 1);
            workspace
                .toast_stack
                .update(ctx, |toasts, ctx| toasts.clear_toasts(ctx));
        });
        source.update(&mut app, |workspace, ctx| {
            workspace.open_settings_pane(None, None, ctx);
            assert_eq!(workspace.tab_count(), 2);
            assert_eq!(workspace.settings_pane.window_id(ctx), source_window_id);
            workspace
                .toast_stack
                .update(ctx, |toasts, ctx| toasts.clear_toasts(ctx));
        });

        settings_view.update(&mut app, |_, ctx| {
            ctx.emit(SettingsViewEvent::ShowToast {
                message: "Settings saved".to_string(),
                flavor: ToastFlavor::Default,
            });
        });

        source.read(&app, |workspace, ctx| {
            assert!(!workspace.toast_stack.as_ref(ctx).has_toasts());
        });
        destination.read(&app, |workspace, ctx| {
            assert!(workspace.toast_stack.as_ref(ctx).has_toasts());
        });
    });
}
