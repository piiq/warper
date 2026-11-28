use super::*;
use crate::elements::Empty;
use pathfinder_geometry::vector::vec2f;

#[derive(Default)]
struct TestView {
    value: usize,
}

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
        "TestView"
    }
}

#[test]
fn failed_window_creation_preserves_existing_window_and_allows_retry() {
    App::test((), |mut app| async move {
        let (source_window, source_view) =
            app.add_window(WindowStyle::NotStealFocus, |_| TestView::default());
        let next_bounds = RectF::new(vec2f(40., 50.), vec2f(800., 600.));

        app.update(|ctx| {
            ctx.next_window_bounds = Some(next_bounds);
            let view_count = ctx.view_to_window.len();
            let presenter_count = ctx.presenters.len();
            let bounds_count = ctx.window_bounds.len();
            let anchor_count = ctx.next_window_bounds_map.len();
            let pending_flushes = ctx.pending_flushes;
            let focused_view = ctx.focused_view_id(source_window);
            ctx.windows().fail_next_open_window_for_test();

            let result = ctx.try_add_window(AddWindowOptions::default(), |_| -> TestView {
                panic!("a failed platform window must not build views or move existing views");
            });

            assert!(result.is_err());
            assert_eq!(ctx.window_ids().collect::<Vec<_>>(), vec![source_window]);
            assert_eq!(ctx.view_to_window.len(), view_count);
            assert_eq!(ctx.presenters.len(), presenter_count);
            assert_eq!(ctx.window_bounds.len(), bounds_count);
            assert_eq!(ctx.next_window_bounds_map.len(), anchor_count);
            assert_eq!(ctx.pending_flushes, pending_flushes);
            assert_eq!(ctx.next_window_bounds, Some(next_bounds));
            assert_eq!(ctx.focused_view_id(source_window), focused_view);
            assert!(ctx.windows().platform_window(source_window).is_some());
        });

        source_view.update(&mut app, |view, ctx| {
            view.value += 1;
            ctx.notify();
        });
        source_view.read(&app, |view, _| assert_eq!(view.value, 1));

        let (destination, destination_view) = app.update(|ctx| {
            ctx.try_add_window(AddWindowOptions::default(), |_| TestView { value: 2 })
                .expect("the next attempt should reach the platform window manager")
        });
        assert_ne!(destination, source_window);
        assert_eq!(app.window_ids().len(), 2);
        destination_view.read(&app, |view, _| assert_eq!(view.value, 2));
        app.read(|ctx| {
            assert_eq!(ctx.next_window_bounds, None);
            assert_eq!(ctx.pending_flushes, 0);
        });
    });
}
