use super::*;
use crate::root_view;
use crate::terminal::ssh::util::InteractiveSshCommand;
use crate::terminal::view::ssh_file_upload::{FileUpload, FileUploadAction};
use warp_core::command::ExitCode;

fn start_upload(
    workspace: &ViewHandle<Workspace>,
    group: &ViewHandle<PaneGroup>,
    app: &mut App,
) -> (RemoteUploadId, TerminalPaneId, ViewHandle<FileUpload>) {
    let (remote_id, upload_view) = group.read(app, |group, ctx| {
        let pane_id = group.pane_id_by_index(0).unwrap();
        let terminal = group.terminal_view_from_pane_id(pane_id, ctx).unwrap();
        (
            pane_id.as_terminal_pane_id().unwrap(),
            terminal.as_ref(ctx).ssh_file_upload().clone(),
        )
    });
    let upload_id = upload_view.update(app, |upload, ctx| {
        upload
            .start_file_upload(
                "example.invalid",
                &["/test-upload.txt".to_string()],
                &None,
                &InteractiveSshCommand::default(),
                ctx,
            )
            .0
    });
    let remote_upload = (remote_id, upload_id);
    let local_id = workspace.read(app, |workspace, _| {
        workspace.file_upload_sessions.upload_id_to_local_map[&remote_upload]
    });
    (remote_upload, local_id, upload_view)
}

fn assert_upload_mapping(
    sessions: &FileUploadSessions,
    remote_upload: RemoteUploadId,
    local: TerminalPaneId,
) {
    assert_eq!(sessions.local_to_remote_map.len(), 1);
    assert_eq!(sessions.local_to_upload_id_map.len(), 1);
    assert_eq!(sessions.upload_id_to_local_map.len(), 1);
    assert_eq!(sessions.local_to_remote_map[&local], remote_upload.0);
    assert_eq!(sessions.local_to_upload_id_map[&local], remote_upload);
    assert_eq!(sessions.upload_id_to_local_map[&remote_upload], local);
}

#[test]
fn moved_tab_upload_controls_and_completion_target_its_inactive_pane_group() {
    let _upload_guard = FeatureFlag::SshDragAndDrop.override_enabled(true);
    App::test((), |mut app| async move {
        initialize_app(&mut app);
        app.update(root_view::init);
        let source = mock_workspace(&mut app);
        let source_window = source.read(&app, |workspace, _| workspace.window_id);
        let (moved_group, retained_group) = source.update(&mut app, |workspace, ctx| {
            let moved_group = workspace.active_tab_pane_group().clone();
            workspace.add_terminal_tab(ctx);
            (moved_group, workspace.active_tab_pane_group().clone())
        });
        let (moved_upload, moved_local, upload_view) =
            start_upload(&source, &moved_group, &mut app);
        let (retained_upload, retained_local, _) = start_upload(&source, &retained_group, &mut app);

        source.update(&mut app, |workspace, ctx| {
            workspace.handle_action(&WorkspaceAction::MoveTabToNewWindow(0), ctx);
        });
        let destination_window = app
            .window_ids()
            .into_iter()
            .find(|window| *window != source_window)
            .unwrap();
        let destination =
            app.update(|ctx| root_view::workspace_for_window(destination_window, ctx).unwrap());
        source.read(&app, |workspace, _| {
            assert_upload_mapping(
                &workspace.file_upload_sessions,
                retained_upload,
                retained_local,
            );
        });
        destination.update(&mut app, |workspace, ctx| {
            assert_upload_mapping(&workspace.file_upload_sessions, moved_upload, moved_local);
            workspace.add_terminal_tab(ctx);
            assert_eq!(workspace.active_tab_index(), 1);
        });
        moved_group.update(&mut app, |_, ctx| {
            ctx.emit(pane_group::Event::FileUploadPasswordPending {
                local_pane_id: moved_local,
            });
        });
        assert!(upload_view.read(&app, |upload, _| upload.has_in_progress_upload()));

        upload_view.update(&mut app, |upload, ctx| {
            upload.handle_action(&FileUploadAction::OpenUploadSession(moved_upload.1), ctx);
        });
        moved_group.read(&app, |group, _| assert_eq!(group.visible_pane_count(), 2));
        destination.read(&app, |workspace, _| {
            assert_eq!(workspace.active_tab_index(), 1)
        });

        moved_group.update(&mut app, |_, ctx| {
            ctx.emit(pane_group::Event::FileUploadFinished {
                local_pane_id: moved_local,
                exit_code: ExitCode::from(0),
            });
        });
        upload_view.read(&app, |upload, _| {
            assert!(upload.has_upload());
            assert!(!upload.has_in_progress_upload());
        });
        upload_view.update(&mut app, |upload, ctx| {
            upload.handle_action(
                &FileUploadAction::TerminateUploadSession(moved_upload.1),
                ctx,
            );
        });
        moved_group.read(&app, |group, _| assert_eq!(group.visible_pane_count(), 1));
        assert!(!upload_view.read(&app, |upload, _| upload.has_upload()));
        source.read(&app, |workspace, _| {
            assert_upload_mapping(
                &workspace.file_upload_sessions,
                retained_upload,
                retained_local,
            );
        });
    });
}
