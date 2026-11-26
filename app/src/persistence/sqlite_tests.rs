use std::{path::PathBuf, sync::Arc};

use diesel::prelude::*;

use crate::{
    app_state::{
        AppState, LeafContents, LeafSnapshot, PaneNodeSnapshot, TabSnapshot, TerminalPaneSnapshot,
        WindowSnapshot,
    },
    persistence::{BlockCompleted, ModelEvent},
    tab::SelectedTabColor,
    terminal::model::block::SerializedBlock,
    terminal::ShellLaunchData,
};

use super::{deduplicate_events, read_sqlite_data, save_app_state, setup_database};

#[test]
fn test_deduplicate_snapshots() {
    let completed_block_1 = BlockCompleted {
        pane_id: vec![1, 2, 3],
        block: Arc::new(SerializedBlock::default()),
        is_local: true,
    };
    let completed_block_2 = BlockCompleted {
        pane_id: vec![4, 5, 6],
        block: Arc::new(SerializedBlock::default()),
        is_local: true,
    };
    let snapshot_1 = AppState {
        active_window_index: Some(1),
        block_lists: Default::default(),
        windows: Default::default(),
    };
    let snapshot_2 = AppState {
        active_window_index: Some(2),
        block_lists: Default::default(),
        windows: Default::default(),
    };
    let snapshot_3 = AppState {
        active_window_index: Some(3),
        block_lists: Default::default(),
        windows: Default::default(),
    };

    let original_events = vec![
        ModelEvent::DeleteBlocks(vec![1, 2, 3]),
        ModelEvent::Snapshot(snapshot_1.clone()),
        ModelEvent::SaveBlock(completed_block_1.clone()),
        ModelEvent::Snapshot(snapshot_2.clone()),
        ModelEvent::SaveBlock(completed_block_2.clone()),
        ModelEvent::Snapshot(snapshot_3.clone()),
        ModelEvent::DeleteBlocks(vec![4, 5, 6]),
    ];

    let filtered_events = deduplicate_events(original_events);
    assert_eq!(filtered_events.len(), 5);

    assert!(matches!(&filtered_events[0], &ModelEvent::DeleteBlocks(_)));
    // The first snapshot should have been filtered out.
    assert!(matches!(&filtered_events[1], &ModelEvent::SaveBlock(_)));
    // The second snapshot should have been filtered out.
    assert!(matches!(&filtered_events[2], &ModelEvent::SaveBlock(_)));
    // The third snapshot should be preserved.
    match &filtered_events[3] {
        ModelEvent::Snapshot(snapshot) => assert_eq!(snapshot, &snapshot_3),
        other => panic!("Expected ModelEvent::Snapshot, got {other:?}"),
    }
    assert!(matches!(&filtered_events[4], &ModelEvent::DeleteBlocks(_)));
}

#[test]
fn test_deduplicate_no_snapshots() {
    let original_events = vec![ModelEvent::SaveBlock(BlockCompleted {
        pane_id: vec![1, 2, 3],
        block: Default::default(),
        is_local: true,
    })];
    let filtered_events = deduplicate_events(original_events);
    assert_eq!(filtered_events.len(), 1);
    assert!(matches!(&filtered_events[0], &ModelEvent::SaveBlock(_)));
}

fn test_terminal_window_snapshot(vertical_tabs_panel_open: bool) -> WindowSnapshot {
    WindowSnapshot {
        tabs: vec![TabSnapshot {
            custom_title: None,
            root: PaneNodeSnapshot::Leaf(LeafSnapshot {
                is_focused: true,
                custom_vertical_tabs_title: None,
                contents: LeafContents::Terminal(TerminalPaneSnapshot {
                    uuid: vec![u8::from(vertical_tabs_panel_open) + 1],
                    cwd: Some("/tmp".to_string()),
                    shell_launch_data: Some(ShellLaunchData::Executable {
                        executable_path: PathBuf::from("/bin/zsh"),
                        shell_type: crate::terminal::shell::ShellType::Zsh,
                    }),
                    is_active: true,
                    is_read_only: false,
                }),
            }),
            default_directory_color: None,
            selected_color: SelectedTabColor::default(),
        }],
        active_tab_index: 0,
        bounds: None,
        fullscreen_state: Default::default(),
        quake_mode: false,
        universal_search_width: None,
        vertical_tabs_panel_open,
    }
}

#[test]
fn test_sqlite_round_trips_vertical_tabs_panel_open() {
    let tempdir = tempfile::tempdir().expect("tempdir should be created");
    let database_path = tempdir.path().join("warp.sqlite");
    let mut conn = setup_database(&database_path).expect("database should initialize");

    let app_state = AppState {
        windows: vec![
            test_terminal_window_snapshot(false),
            test_terminal_window_snapshot(true),
        ],
        active_window_index: Some(1),
        block_lists: Default::default(),
    };

    save_app_state(&mut conn, &app_state).expect("app state should save");

    let restored = read_sqlite_data(&mut conn)
        .expect("app state should load")
        .app_state;

    assert_eq!(restored.active_window_index, Some(1));
    assert_eq!(
        restored
            .windows
            .iter()
            .map(|window| window.vertical_tabs_panel_open)
            .collect::<Vec<_>>(),
        vec![false, true]
    );
}

#[test]
fn test_sqlite_discards_removed_review_pane_and_preserves_terminal() {
    use crate::app_state::{BranchSnapshot, PaneFlex, SplitDirection};
    use crate::persistence::schema;

    let tempdir = tempfile::tempdir().expect("tempdir should be created");
    let mut conn =
        setup_database(&tempdir.path().join("warp.sqlite")).expect("database should initialize");
    let mut window = test_terminal_window_snapshot(false);
    let terminal = window.tabs[0].root.clone();
    let mut obsolete_pane = terminal.clone();
    let PaneNodeSnapshot::Leaf(leaf) = &mut obsolete_pane else {
        panic!("expected terminal leaf");
    };
    let LeafContents::Terminal(snapshot) = &mut leaf.contents else {
        panic!("expected terminal contents");
    };
    snapshot.uuid = vec![2];
    window.tabs[0].root = PaneNodeSnapshot::Branch(BranchSnapshot {
        direction: SplitDirection::Horizontal,
        children: vec![
            (PaneFlex(1.), terminal.clone()),
            (PaneFlex(1.), obsolete_pane),
        ],
    });
    save_app_state(
        &mut conn,
        &AppState {
            windows: vec![window],
            active_window_index: Some(0),
            block_lists: Default::default(),
        },
    )
    .expect("app state should save");

    // Recreate a review leaf saved by an older version of Warper.
    let obsolete_id: i32 = schema::terminal_panes::table
        .filter(schema::terminal_panes::uuid.eq(vec![2u8]))
        .select(schema::terminal_panes::id)
        .first(&mut conn)
        .unwrap();
    diesel::delete(schema::terminal_panes::table.find(obsolete_id))
        .execute(&mut conn)
        .unwrap();
    diesel::update(
        schema::pane_leaves::table.filter(schema::pane_leaves::pane_node_id.eq(obsolete_id)),
    )
    .set(schema::pane_leaves::kind.eq("code_review"))
    .execute(&mut conn)
    .unwrap();
    diesel::insert_into(schema::code_review_panes::table)
        .values((
            schema::code_review_panes::id.eq(obsolete_id),
            schema::code_review_panes::terminal_uuid.eq(vec![1u8]),
            schema::code_review_panes::repo_path.eq("/tmp"),
        ))
        .execute(&mut conn)
        .unwrap();

    let restored = read_sqlite_data(&mut conn).unwrap().app_state;
    assert_eq!(restored.windows.len(), 1);
    assert_eq!(restored.windows[0].tabs.len(), 1);
    let PaneNodeSnapshot::Branch(branch) = &restored.windows[0].tabs[0].root else {
        panic!("expected restored split");
    };
    assert_eq!(branch.children.len(), 1);
    assert_eq!(branch.children[0].1, terminal);
}

#[test]
fn test_sqlite_round_trips_custom_vertical_tabs_title() {
    let tempdir = tempfile::tempdir().expect("tempdir should be created");
    let database_path = tempdir.path().join("warp.sqlite");
    let mut conn = setup_database(&database_path).expect("database should initialize");

    let app_state = AppState {
        windows: vec![WindowSnapshot {
            tabs: vec![TabSnapshot {
                custom_title: None,
                root: PaneNodeSnapshot::Leaf(LeafSnapshot {
                    is_focused: true,
                    custom_vertical_tabs_title: Some("Production API".to_string()),
                    contents: LeafContents::Terminal(TerminalPaneSnapshot {
                        uuid: vec![42],
                        cwd: Some("/tmp".to_string()),
                        shell_launch_data: Some(ShellLaunchData::Executable {
                            executable_path: PathBuf::from("/bin/zsh"),
                            shell_type: crate::terminal::shell::ShellType::Zsh,
                        }),
                        is_active: true,
                        is_read_only: false,
                    }),
                }),
                default_directory_color: None,
                selected_color: SelectedTabColor::default(),
            }],
            active_tab_index: 0,
            bounds: None,
            fullscreen_state: Default::default(),
            quake_mode: false,
            universal_search_width: None,
            vertical_tabs_panel_open: false,
        }],
        active_window_index: Some(0),
        block_lists: Default::default(),
    };

    save_app_state(&mut conn, &app_state).expect("app state should save");

    let restored = read_sqlite_data(&mut conn)
        .expect("app state should load")
        .app_state;

    let PaneNodeSnapshot::Leaf(LeafSnapshot {
        custom_vertical_tabs_title,
        ..
    }) = &restored.windows[0].tabs[0].root
    else {
        panic!("Expected terminal pane leaf");
    };
    assert_eq!(
        custom_vertical_tabs_title.as_deref(),
        Some("Production API")
    );
}
