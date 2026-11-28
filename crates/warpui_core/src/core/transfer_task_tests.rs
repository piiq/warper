use super::*;
use crate::elements::Empty;

#[derive(Default)]
struct TaskView {
    events: Vec<(Option<usize>, WindowId)>,
}

impl Entity for TaskView {
    type Event = ();
}

impl TypedActionView for TaskView {
    type Action = ();
}

impl View for TaskView {
    fn render(&self, _: &AppContext) -> Box<dyn Element> {
        Empty::new().finish()
    }

    fn ui_name() -> &'static str {
        "TaskView"
    }
}

#[test]
fn transferred_view_receives_pending_future_after_source_closes() {
    App::test((), |mut app| async move {
        let (source, _) = app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let (destination, _) = app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let view = app.add_view(source, |_| TaskView::default());
        let (release, pending) = futures::channel::oneshot::channel();
        view.update(&mut app, |_, ctx| {
            ctx.spawn(async move { pending.await.unwrap() }, |view, value, ctx| {
                view.events.push((Some(value), ctx.window_id()));
                ctx.notify();
            });
        });
        app.update(|ctx| {
            assert!(ctx.transfer_view_to_window(view.id(), source, destination));
            ctx.handle_window_closed(source).unwrap();
        });

        release.send(7).unwrap();
        app.finish_pending_tasks().await;

        view.read(&app, |view, _| {
            assert_eq!(view.events, vec![(Some(7), destination)]);
        });
    });
}

#[test]
fn transferred_view_receives_stream_items_and_completion_in_current_window() {
    App::test((), |mut app| async move {
        let (source, _) = app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let (destination, _) = app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let view = app.add_view(source, |_| TaskView::default());
        let (items, stream) = async_channel::unbounded();
        let completion = view.update(&mut app, |_, ctx| {
            ctx.spawn_stream_local(
                stream,
                |view, value, ctx| {
                    view.events.push((Some(value), ctx.window_id()));
                    ctx.notify();
                },
                |view, ctx| {
                    view.events.push((None, ctx.window_id()));
                    ctx.notify();
                },
            )
            .into_future()
        });
        app.update(|ctx| {
            assert!(ctx.transfer_view_to_window(view.id(), source, destination));
            ctx.handle_window_closed(source).unwrap();
        });

        items.send(3).await.unwrap();
        items.send(5).await.unwrap();
        drop(items);
        completion.await;

        view.read(&app, |view, _| {
            assert_eq!(
                view.events,
                vec![
                    (Some(3), destination),
                    (Some(5), destination),
                    (None, destination),
                ],
            );
        });
    });
}

#[test]
fn transferred_view_receives_empty_stream_completion_after_source_closes() {
    App::test((), |mut app| async move {
        let (source, _) = app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let (destination, _) = app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let view = app.add_view(source, |_| TaskView::default());
        let (items, stream) = async_channel::unbounded::<usize>();
        let completion = view.update(&mut app, |_, ctx| {
            ctx.spawn_stream_local(
                stream,
                |_, _, _| panic!("this stream must complete without yielding an item"),
                |view, ctx| view.events.push((None, ctx.window_id())),
            )
            .into_future()
        });
        app.update(|ctx| {
            assert!(ctx.transfer_view_to_window(view.id(), source, destination));
            ctx.handle_window_closed(source).unwrap();
        });

        drop(items);
        completion.await;

        view.read(&app, |view, _| {
            assert_eq!(view.events, vec![(None, destination)]);
        });
    });
}

#[test]
fn transfer_includes_unrendered_untyped_and_optional_children() {
    App::test((), |mut app| async move {
        let (source, source_root) =
            app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let (destination, destination_root) =
            app.add_window(WindowStyle::NotStealFocus, |_| TaskView::default());
        let parent = source_root.update(&mut app, |_, ctx| {
            ctx.add_typed_action_view(|_| TaskView::default())
        });
        let untyped_child = parent.update(&mut app, |_, ctx| ctx.add_view(|_| TaskView::default()));
        let optional_child = untyped_child.update(&mut app, |_, ctx| {
            ctx.add_option_view(|_| Some(TaskView::default())).unwrap()
        });
        parent.update(&mut app, |_, ctx| {
            assert!(ctx.add_option_view::<TaskView, _>(|_| None).is_none());
        });
        let unrelated = app.add_view(source, |_| TaskView::default());

        app.update(|ctx| {
            let transferred = ctx.transfer_view_tree_to_window(parent.id(), source, destination);
            assert_eq!(transferred.len(), 3);
            for view in [&parent, &untyped_child, &optional_child] {
                assert!(transferred.contains(&view.id()));
                assert_eq!(view.window_id(ctx), destination);
            }
            assert_eq!(unrelated.window_id(ctx), source);
            ctx.set_view_parent(parent.id(), destination_root.id());
            assert_eq!(
                ctx.structural_child_to_parent.get(&parent.id()),
                Some(&destination_root.id()),
            );
            assert!(!ctx
                .structural_parent_to_children
                .contains_key(&source_root.id()));
            assert!(
                ctx.structural_parent_to_children[&destination_root.id()].contains(&parent.id())
            );
            ctx.handle_window_closed(source).unwrap();
        });

        optional_child.update(&mut app, |view, ctx| {
            view.events.push((Some(11), ctx.window_id()));
            ctx.notify();
        });
        optional_child.read(&app, |view, _| {
            assert_eq!(view.events, vec![(Some(11), destination)]);
        });
    });
}
