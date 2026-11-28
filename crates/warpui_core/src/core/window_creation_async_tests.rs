use std::cell::Cell;

use super::*;
use crate::elements::Empty;

struct TestView;

impl Entity for TestView {
    type Event = ();
}

impl TypedActionView for TestView {
    type Action = ();
}

impl View for TestView {
    fn render(&self, _: &AppContext) -> Box<dyn Element> {
        Empty::new().finish()
    }

    fn ui_name() -> &'static str {
        "AsyncWindowTestView"
    }
}

#[test]
fn asynchronous_window_waits_for_native_success_and_flushes_after_completion() {
    App::test((), |mut app| async move {
        let (source, _) = app.add_window(WindowStyle::NotStealFocus, |_| TestView);
        let completed = Rc::new(Cell::new(false));
        let effect_ran = Rc::new(Cell::new(false));
        let built = Rc::new(Cell::new(false));
        let pending = app.update(|ctx| {
            let completed_for_effect = completed.clone();
            let effect_ran = effect_ran.clone();
            ctx.add_global_action("test:window_built", move |_: &(), _| {
                assert!(completed_for_effect.get());
                effect_ran.set(true);
            });
            ctx.defer_next_window_opening = true;
            let built = built.clone();
            let completed = completed.clone();
            ctx.try_add_window_async(
                AddWindowOptions::default(),
                move |window_id, ctx| {
                    built.set(true);
                    Ok(ctx.add_typed_action_view(window_id, |ctx| {
                        ctx.dispatch_global_action("test:window_built", ());
                        TestView
                    }))
                },
                move |result, ctx| {
                    let (window_id, root) = result.unwrap();
                    assert_ne!(window_id, source);
                    assert_eq!(ctx.root_view_id(window_id), Some(root.id()));
                    assert_eq!(ctx.window_ids().count(), 2);
                    completed.set(true);
                },
            );
            assert_eq!(ctx.window_ids().collect::<Vec<_>>(), vec![source]);
            assert_eq!(ctx.pending_window_openings.len(), 1);
            *ctx.pending_window_openings.keys().next().unwrap()
        });
        assert!(!built.get());
        assert!(!completed.get());
        assert!(!effect_ran.get());

        app.update(|ctx| {
            let window = ctx.windows().platform_window(pending).unwrap();
            assert!((window.callbacks().opened_callback)(Ok(()), ctx));
        });

        assert!(built.get());
        assert!(completed.get());
        assert!(effect_ran.get());
        app.read(|ctx| {
            assert!(ctx.pending_window_openings.is_empty());
            assert_eq!(ctx.pending_flushes, 0);
        });
    });
}

#[test]
fn asynchronous_native_failure_keeps_source_and_allows_retry() {
    App::test((), |mut app| async move {
        let (source, source_view) = app.add_window(WindowStyle::NotStealFocus, |_| TestView);
        let failed = Rc::new(Cell::new(false));
        let pending = app.update(|ctx| {
            ctx.next_window_bounds = Some(RectF::new(Vector2F::splat(20.), Vector2F::splat(500.)));
            ctx.defer_next_window_opening = true;
            let failed = failed.clone();
            ctx.try_add_window_async(
                AddWindowOptions::default(),
                |_, _| -> Result<ViewHandle<TestView>> {
                    panic!("Native failure must not build or move views")
                },
                move |result, ctx| {
                    assert!(result.is_err());
                    assert_eq!(ctx.window_ids().collect::<Vec<_>>(), vec![source]);
                    failed.set(true);
                },
            );
            *ctx.pending_window_openings.keys().next().unwrap()
        });

        app.update(|ctx| {
            let next_bounds = ctx.next_window_bounds;
            let window = ctx.windows().platform_window(pending).unwrap();
            assert!((window.callbacks().opened_callback)(
                Err(anyhow!("Native creation failed")),
                ctx
            ));
            assert!(ctx.windows().platform_window(pending).is_none());
            assert!(!ctx.window_bounds.contains_key(&pending));
            assert!(!ctx.presenters.contains_key(&pending));
            assert!(!ctx.next_window_bounds_map.contains_key(&pending));
            assert_eq!(ctx.next_window_bounds, next_bounds);
            assert_eq!(source_view.window_id(ctx), source);
            assert!(ctx.view_to_window.contains_key(&source_view.id()));
        });
        assert!(failed.get());

        let retried = Rc::new(Cell::new(false));
        app.update(|ctx| {
            let retried = retried.clone();
            ctx.try_add_window_async(
                AddWindowOptions::default(),
                |window_id, ctx| Ok(ctx.add_typed_action_view(window_id, |_| TestView)),
                move |result, _| {
                    assert!(result.is_ok());
                    retried.set(true);
                },
            );
        });
        assert!(retried.get());
        assert_eq!(app.window_ids().len(), 2);
        app.read(|ctx| assert_eq!(ctx.pending_flushes, 0));
    });
}

#[test]
fn asynchronous_builder_rejection_discards_window_without_close_callback() {
    App::test((), |mut app| async move {
        let (source, _) = app.add_window(WindowStyle::NotStealFocus, |_| TestView);
        let rejected = Rc::new(Cell::new(false));
        let pending = app.update(|ctx| {
            ctx.defer_next_window_opening = true;
            let rejected = rejected.clone();
            ctx.try_add_window_async(
                AddWindowOptions::default(),
                |window_id, ctx| -> Result<ViewHandle<TestView>> {
                    let _partial_view = ctx.add_typed_action_view(window_id, |_| TestView);
                    Err(anyhow!("The source tab no longer exists"))
                },
                move |result, _| {
                    assert!(result.is_err());
                    rejected.set(true);
                },
            );
            *ctx.pending_window_openings.keys().next().unwrap()
        });
        app.update(|ctx| {
            let window = ctx.windows().platform_window(pending).unwrap();
            assert!((window.callbacks().opened_callback)(Ok(()), ctx));
        });
        assert!(rejected.get());
        assert_eq!(app.window_ids(), vec![source]);
        app.read(|ctx| {
            assert!(ctx.view_to_window.values().all(|window| *window == source));
            assert!(ctx.windows().platform_window(pending).is_none());
            assert!(ctx.pending_window_openings.is_empty());
            assert_eq!(ctx.pending_flushes, 0);
        });

        let mut dispatcher = platform::app::AppCallbackDispatcher::new(
            platform::app::AppCallbacks {
                on_window_will_close: Some(Box::new(|_, _| {
                    panic!("A rejected opening must not enter the normal close/undo path")
                })),
                ..Default::default()
            },
            app.clone(),
        );
        dispatcher.window_will_close(pending);
    });
}
