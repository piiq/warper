use super::*;
use crate::elements::Empty;

struct Emitter;

impl Entity for Emitter {
    type Event = usize;
}

#[derive(Default)]
struct Subscriber {
    events: Vec<usize>,
}

impl Entity for Subscriber {
    type Event = usize;
}

impl TypedActionView for Subscriber {
    type Action = ();
}

impl View for Subscriber {
    fn render(&self, _: &AppContext) -> Box<dyn Element> {
        Empty::new().finish()
    }

    fn ui_name() -> &'static str {
        "Subscriber"
    }
}

#[test]
fn transferred_view_unsubscribes_from_model_and_view_events() {
    App::test((), |mut app| async move {
        let (source, emitter_view) =
            app.add_window(WindowStyle::NotStealFocus, |_| Subscriber::default());
        let (destination, _) =
            app.add_window(WindowStyle::NotStealFocus, |_| Subscriber::default());
        let emitter_model = app.add_model(|_| Emitter);
        let moved = app.add_view(source, |_| Subscriber::default());
        let unrelated = app.add_view(source, |_| Subscriber::default());
        for view in [&moved, &unrelated] {
            view.update(&mut app, |_, ctx| {
                ctx.subscribe_to_model(&emitter_model, |view, _, value, _| {
                    view.events.push(*value);
                });
                ctx.subscribe_to_view(&emitter_view, |view, _, value, _| {
                    view.events.push(*value);
                });
            });
        }
        emitter_model.update(&mut app, |_, ctx| ctx.emit(1));
        emitter_view.update(&mut app, |_, ctx| ctx.emit(10));
        app.update(|ctx| {
            assert!(ctx.transfer_view_to_window(moved.id(), source, destination));
        });

        moved.update(&mut app, |_, ctx| {
            ctx.unsubscribe_to_model(&emitter_model);
            ctx.unsubscribe_to_view(&emitter_view);
        });
        emitter_model.update(&mut app, |_, ctx| ctx.emit(2));
        emitter_view.update(&mut app, |_, ctx| ctx.emit(20));
        moved.read(&app, |view, _| assert_eq!(view.events, vec![1, 10]));

        moved.update(&mut app, |_, ctx| {
            ctx.subscribe_to_model(&emitter_model, |view, _, value, _| {
                view.events.push(*value);
            });
            ctx.subscribe_to_view(&emitter_view, |view, _, value, _| {
                view.events.push(*value);
            });
        });
        emitter_model.update(&mut app, |_, ctx| ctx.emit(3));
        emitter_view.update(&mut app, |_, ctx| ctx.emit(30));
        moved.read(&app, |view, _| assert_eq!(view.events, vec![1, 10, 3, 30]));
        unrelated.read(&app, |view, _| {
            assert_eq!(view.events, vec![1, 10, 2, 20, 3, 30]);
        });
    });
}

#[test]
fn transferred_view_unsubscribes_inside_model_event_before_resubscribing() {
    App::test((), |mut app| async move {
        let (source, _) = app.add_window(WindowStyle::NotStealFocus, |_| Subscriber::default());
        let (destination, _) =
            app.add_window(WindowStyle::NotStealFocus, |_| Subscriber::default());
        let emitter = app.add_model(|_| Emitter);
        let moved = app.add_view(source, |_| Subscriber::default());
        let unrelated = app.add_view(source, |_| Subscriber::default());
        moved.update(&mut app, |_, ctx| {
            ctx.subscribe_to_model(&emitter, |view, emitter, value, ctx| {
                view.events.push(*value);
                ctx.unsubscribe_to_model(&emitter);
                ctx.subscribe_to_model(&emitter, |view, _, value, _| {
                    view.events.push(*value * 100);
                });
            });
        });
        unrelated.update(&mut app, |_, ctx| {
            ctx.subscribe_to_model(&emitter, |view, _, value, _| {
                view.events.push(*value);
            });
        });
        app.update(|ctx| {
            assert!(ctx.transfer_view_to_window(moved.id(), source, destination));
        });

        emitter.update(&mut app, |_, ctx| ctx.emit(1));
        emitter.update(&mut app, |_, ctx| ctx.emit(2));
        emitter.update(&mut app, |_, ctx| ctx.emit(3));

        moved.read(&app, |view, _| assert_eq!(view.events, vec![1, 200, 300]));
        unrelated.read(&app, |view, _| assert_eq!(view.events, vec![1, 2, 3]));
    });
}

#[test]
fn transferred_view_unsubscribes_inside_view_event_before_resubscribing() {
    App::test((), |mut app| async move {
        let (source, emitter) =
            app.add_window(WindowStyle::NotStealFocus, |_| Subscriber::default());
        let (destination, _) =
            app.add_window(WindowStyle::NotStealFocus, |_| Subscriber::default());
        let moved = app.add_view(source, |_| Subscriber::default());
        let unrelated = app.add_view(source, |_| Subscriber::default());
        moved.update(&mut app, |_, ctx| {
            ctx.subscribe_to_view(&emitter, |view, emitter, value, ctx| {
                view.events.push(*value);
                ctx.unsubscribe_to_view(&emitter);
                ctx.subscribe_to_view(&emitter, |view, _, value, _| {
                    view.events.push(*value * 100);
                });
            });
        });
        unrelated.update(&mut app, |_, ctx| {
            ctx.subscribe_to_view(&emitter, |view, _, value, _| {
                view.events.push(*value);
            });
        });
        app.update(|ctx| {
            assert!(ctx.transfer_view_to_window(moved.id(), source, destination));
        });

        emitter.update(&mut app, |_, ctx| ctx.emit(1));
        emitter.update(&mut app, |_, ctx| ctx.emit(2));
        emitter.update(&mut app, |_, ctx| ctx.emit(3));

        moved.read(&app, |view, _| assert_eq!(view.events, vec![1, 200, 300]));
        unrelated.read(&app, |view, _| assert_eq!(view.events, vec![1, 2, 3]));
    });
}
