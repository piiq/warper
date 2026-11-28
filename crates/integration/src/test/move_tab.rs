use std::sync::Arc;

use pathfinder_geometry::vector::vec2f;
use warp::{
    integration_testing::{
        input::input_contains_string,
        step::new_step_with_default_assertions,
        terminal::{
            assert_active_block_received_precmd,
            assert_command_executed_for_single_terminal_in_tab, assert_focused_editor_in_tab,
            assert_long_running_block_executing_for_single_terminal_in_tab,
            execute_command_for_single_terminal_in_tab, execute_long_running_command,
            util::ExpectedExitStatus, validate_block_output_on_finished_block,
            wait_until_bootstrapped_single_pane_for_tab,
        },
        view_getters::{single_terminal_view_for_tab, workspace_view},
        workspace::assert_tab_count,
    },
    terminal::{model::BlockId, TerminalView},
    workspace::{TabContextMenuAnchor, WorkspaceAction, NEW_TAB_BUTTON_POSITION_ID},
};
use warpui::{
    async_assert, async_assert_eq, integration::TestStep, integration_assert,
    platform::TerminationMode, TypedActionView, ViewHandle, WindowId,
};

use crate::{util::skip_if_powershell_core_2303, Builder};

use super::new_builder;

pub fn test_move_tab_to_new_window_preserves_session() -> Builder {
    new_builder()
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open a second tab")
                .with_click_on_saved_position(NEW_TAB_BUTTON_POSITION_ID)
                .add_assertion(assert_tab_count(2)),
        )
        .with_step(wait_until_bootstrapped_single_pane_for_tab(1))
        .with_step(execute_command_for_single_terminal_in_tab(
            1,
            "echo before-transfer".to_string(),
            ExpectedExitStatus::Success,
            "before-transfer".to_string(),
        ))
        .with_step(
            new_step_with_default_assertions("Leave a pending command in the tab")
                .with_typed_characters(&["echo moved"])
                .add_assertion(input_contains_string(1, "echo moved".to_string()))
                .with_action(|app, window_id, data| {
                    let terminal = single_terminal_view_for_tab(app, window_id, 1);
                    data.insert("source_window", window_id);
                    data.insert(
                        "terminal_model",
                        terminal.read(app, |view, _| view.model.clone()),
                    );
                    data.insert("terminal", terminal);
                }),
        )
        .with_step(
            new_step_with_default_assertions("Open the inactive tab's context menu")
                .with_keystrokes(&["cmdorctrl-1"])
                .with_action(|app, window_id, _| {
                    workspace_view(app, window_id).update(app, |workspace, ctx| {
                        workspace.toggle_tab_right_click_menu(
                            1,
                            TabContextMenuAnchor::Pointer(vec2f(150., 30.)),
                            ctx,
                        );
                        assert!(
                            workspace.select_tab_context_menu_item("Move Tab to New Window", ctx)
                        );
                    });
                }),
        )
        .with_step(
            TestStep::new("Activate Move Tab to New Window with Enter")
                .with_keystrokes(&["enter"])
                .add_named_assertion("exactly two windows", |app, _| {
                    async_assert_eq!(app.window_ids().len(), 2)
                })
                .add_named_assertion_with_data_from_prior_step(
                    "destination is active",
                    |app, _, data| {
                        let source_window: &WindowId = data.get("source_window").unwrap();
                        app.read(|ctx| {
                            let active_window = ctx.windows().active_window();
                            async_assert!(
                                active_window.is_some() && active_window != Some(*source_window)
                            )
                        })
                    },
                ),
        )
        .with_step(
            new_step_with_default_assertions("Check the moved session and keyboard focus")
                .add_assertion(assert_tab_count(1))
                .add_assertion(assert_focused_editor_in_tab(0))
                .add_assertion(input_contains_string(0, "echo moved".to_string()))
                .add_named_assertion_with_data_from_prior_step(
                    "live terminal and source selection are preserved",
                    |app, window_id, data| {
                        let source_window: &WindowId = data.get("source_window").unwrap();
                        integration_assert!(window_id != *source_window);
                        let original: &ViewHandle<TerminalView> = data.get("terminal").unwrap();
                        let terminal = single_terminal_view_for_tab(app, window_id, 0);
                        integration_assert!(terminal.id() == original.id());
                        let model = terminal.read(app, |view, _| view.model.clone());
                        integration_assert!(Arc::ptr_eq(
                            &model,
                            data.get("terminal_model").unwrap()
                        ));
                        workspace_view(app, *source_window).read(app, |workspace, _| {
                            async_assert!(
                                workspace.tab_count() == 1 && workspace.active_tab_index() == 0
                            )
                        })
                    },
                ),
        )
        .with_step(
            TestStep::new("Close the source window")
                .with_action(|app, _, data| {
                    let source_window: &WindowId = data.get("source_window").unwrap();
                    app.update(|ctx| {
                        ctx.windows()
                            .close_window(*source_window, TerminationMode::ForceTerminate);
                    });
                })
                .add_named_assertion("destination stays open", |app, _| {
                    async_assert_eq!(app.window_ids().len(), 1)
                }),
        )
        .with_step(
            new_step_with_default_assertions("Type and execute in the transferred terminal")
                .with_typed_characters(&["-live"])
                .with_keystrokes(&["enter"])
                .add_assertion(assert_command_executed_for_single_terminal_in_tab(
                    0,
                    "echo moved-live".to_string(),
                ))
                .add_named_assertion(
                    "command output survives source closure",
                    |app, window_id| {
                        validate_block_output_on_finished_block("moved-live", 0, 0, window_id, app)
                    },
                ),
        )
}

pub fn test_move_tab_to_new_window_can_close_destination() -> Builder {
    close_destination_test(
        TestStep::new("Close the moved window through the cancellable close path").with_action(
            |app, window_id, _| {
                app.read(|ctx| {
                    ctx.windows()
                        .close_window(window_id, TerminationMode::Cancellable)
                });
            },
        ),
    )
}

#[cfg(target_os = "macos")]
pub fn test_move_tab_to_new_window_native_close_button() -> Builder {
    close_destination_test(
        TestStep::new("Click the moved window's native close button").with_action(
            |app, window_id, _| {
                use warpui::platform::mac::WindowExt;
                app.read(|ctx| {
                    ctx.windows()
                        .platform_window(window_id)
                        .unwrap()
                        .as_ref()
                        .click_native_close_button();
                });
            },
        ),
    )
    .with_real_display()
    .with_setup(|utils| {
        utils.set_env("WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS", Some("1"));
    })
}

fn close_destination_test(close_step: TestStep) -> Builder {
    new_builder()
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open a second tab")
                .with_click_on_saved_position(NEW_TAB_BUTTON_POSITION_ID)
                .add_assertion(assert_tab_count(2)),
        )
        .with_step(wait_until_bootstrapped_single_pane_for_tab(1))
        .with_step(
            TestStep::new("Move the tab into a new window")
                .with_action(|app, window_id, data| {
                    data.insert("source_window", window_id);
                    workspace_view(app, window_id).update(app, |workspace, ctx| {
                        workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(1), ctx);
                    });
                })
                .add_named_assertion("exactly two windows", |app, _| {
                    async_assert_eq!(app.window_ids().len(), 2)
                })
                .add_named_assertion_with_data_from_prior_step(
                    "destination is active",
                    |app, _, data| {
                        let source: &WindowId = data.get("source_window").unwrap();
                        app.read(|ctx| {
                            async_assert!(ctx
                                .windows()
                                .active_window()
                                .is_some_and(|id| id != *source))
                        })
                    },
                ),
        )
        .with_step(close_step.add_named_assertion_with_data_from_prior_step(
            "source remains open",
            |app, _, data| {
                let source: &WindowId = data.get("source_window").unwrap();
                async_assert_eq!(app.window_ids(), vec![*source])
            },
        ))
        .with_step(execute_command_for_single_terminal_in_tab(
            0,
            "echo source-still-works".to_string(),
            ExpectedExitStatus::Success,
            "source-still-works".to_string(),
        ))
}

pub fn test_move_tab_to_new_window_preserves_running_command() -> Builder {
    new_builder()
        // Use the same shell coverage as test_ctrl_c (CORE-2734).
        .set_should_run_test(skip_if_powershell_core_2303)
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open a second tab")
                .with_click_on_saved_position(NEW_TAB_BUTTON_POSITION_ID)
                .add_assertion(assert_tab_count(2)),
        )
        .with_step(wait_until_bootstrapped_single_pane_for_tab(1))
        .with_step(execute_long_running_command(1, "sleep 999".to_string()))
        .with_step(
            TestStep::new("Capture the running terminal and move its tab")
                .with_action(|app, window_id, data| {
                    let terminal = single_terminal_view_for_tab(app, window_id, 1);
                    data.insert("source_window", window_id);
                    data.insert(
                        "terminal_model",
                        terminal.read(app, |view, _| view.model.clone()),
                    );
                    data.insert(
                        "running_block",
                        terminal.read(app, |view, _| {
                            view.model.lock().block_list().active_block().id().clone()
                        }),
                    );
                    data.insert("terminal", terminal);
                    workspace_view(app, window_id).update(app, |workspace, ctx| {
                        workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(1), ctx);
                    });
                })
                .add_named_assertion("exactly two windows", |app, _| {
                    async_assert_eq!(app.window_ids().len(), 2)
                })
                .add_named_assertion_with_data_from_prior_step(
                    "destination is active",
                    |app, _, data| {
                        let source_window: &WindowId = data.get("source_window").unwrap();
                        app.read(|ctx| {
                            let active_window = ctx.windows().active_window();
                            async_assert!(
                                active_window.is_some() && active_window != Some(*source_window)
                            )
                        })
                    },
                ),
        )
        .with_step(
            TestStep::new("Close the source window while the moved command runs")
                .with_action(|app, _, data| {
                    let source_window: &WindowId = data.get("source_window").unwrap();
                    app.update(|ctx| {
                        ctx.windows()
                            .close_window(*source_window, TerminationMode::ForceTerminate);
                    });
                })
                .add_named_assertion("destination stays open", |app, _| {
                    async_assert_eq!(app.window_ids().len(), 1)
                })
                .add_assertion(assert_tab_count(1))
                .add_assertion(
                    assert_long_running_block_executing_for_single_terminal_in_tab(true, 0),
                )
                .add_named_assertion_with_data_from_prior_step(
                    "the same terminal and command are still running",
                    |app, window_id, data| {
                        let original: &ViewHandle<TerminalView> = data.get("terminal").unwrap();
                        let running_block: &BlockId = data.get("running_block").unwrap();
                        let terminal = single_terminal_view_for_tab(app, window_id, 0);
                        integration_assert!(terminal.id() == original.id());
                        let model = terminal.read(app, |view, _| view.model.clone());
                        integration_assert!(Arc::ptr_eq(
                            &model,
                            data.get("terminal_model").unwrap()
                        ));
                        let model = model.lock();
                        let block = model.block_list().active_block();
                        async_assert!(
                            block.id() == running_block && block.is_active_and_long_running()
                        )
                    },
                ),
        )
        .with_step(
            new_step_with_default_assertions("Interrupt the moved command")
                .with_keystrokes(&["ctrl-c"])
                .add_named_assertion(
                    "shell prompt is ready after interruption",
                    assert_active_block_received_precmd(0, 0),
                )
                .add_assertion(assert_focused_editor_in_tab(0)),
        )
        .with_step(
            new_step_with_default_assertions("Type into the terminal after interruption")
                .with_typed_characters(&["echo after-running-transfer"])
                .add_assertion(input_contains_string(
                    0,
                    "echo after-running-transfer".to_string(),
                )),
        )
        .with_step(
            new_step_with_default_assertions("Execute the command after interruption")
                .with_keystrokes(&["enter"])
                .add_assertion(assert_command_executed_for_single_terminal_in_tab(
                    0,
                    "echo after-running-transfer".to_string(),
                ))
                .add_named_assertion("command output after interruption", |app, window_id| {
                    validate_block_output_on_finished_block(
                        "after-running-transfer",
                        0,
                        0,
                        window_id,
                        app,
                    )
                })
                .add_assertion(assert_active_block_received_precmd(0, 0)),
        )
}
