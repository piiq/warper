use crate::app_state::get_app_state;
use crate::network::NetworkStatus;
use crate::persistence::ModelEvent;
use crate::terminal::alt_screen_reporting::AltScreenReporting;
use crate::terminal::general_settings::GeneralSettings;
use ::settings::ToggleableSetting;

use crate::undo_close::UndoCloseStack;
use crate::GlobalResourceHandlesProvider;
use warpui::{AppContext, SingletonEntity};

/// DEPRECATED. Global actions are being phased out.
/// Do not add any more global actions; use typed actions instead.
pub fn init_global_actions(app: &mut AppContext) {
    app.add_global_action("workspace:toggle_mouse_reporting", toggle_mouse_reporting);
    app.add_global_action("workspace:toggle_scroll_reporting", toggle_scroll_reporting);
    app.add_global_action("workspace:toggle_focus_reporting", toggle_focus_reporting);
    app.add_global_action("workspace:save_app", save_app);
    app.add_global_action(
        "workspace:toggle_debug_network_status",
        toggle_debug_network_status,
    );
    app.add_global_action("app:undo_close", undo_close);
}

fn toggle_mouse_reporting(_: &(), ctx: &mut AppContext) {
    AltScreenReporting::handle(ctx).update(ctx, |reporting, ctx| {
        reporting
            .mouse_reporting_enabled
            .toggle_and_save_value(ctx)
            .expect("MouseReportingEnabled failed to serialize");
    });
}

fn toggle_scroll_reporting(_: &(), ctx: &mut AppContext) {
    AltScreenReporting::handle(ctx).update(ctx, |reporting, ctx| {
        reporting
            .scroll_reporting_enabled
            .toggle_and_save_value(ctx)
            .expect("ScrollReportingEnabled failed to serialize");
    });
}

fn toggle_focus_reporting(_: &(), ctx: &mut AppContext) {
    AltScreenReporting::handle(ctx).update(ctx, |reporting, ctx| {
        reporting
            .focus_reporting_enabled
            .toggle_and_save_value(ctx)
            .expect("FocusReportingEnabled failed to serialize");
    });
}

fn save_app(_: &(), ctx: &mut AppContext) {
    if !*GeneralSettings::as_ref(ctx).restore_session {
        return;
    }

    let Some(model_event_sender) = GlobalResourceHandlesProvider::as_ref(ctx)
        .get()
        .model_event_sender
        .clone()
    else {
        return;
    };

    // Only compute the app state if we're definitely going to use it.
    let app_state = get_app_state(ctx);
    let event = ModelEvent::Snapshot(app_state);

    if let Err(err) = model_event_sender.send(event) {
        log::error!("Error trying to send model event {err:?}");
    }
}

fn toggle_debug_network_status(_: &(), ctx: &mut AppContext) {
    NetworkStatus::handle(ctx).update(ctx, move |me, ctx| {
        let is_reachable = me.is_online();
        let new_is_reachable = !is_reachable;
        if new_is_reachable {
            log::info!("Manually toggled network status to be reachable");
        } else {
            log::info!("Manually toggled network status to be not reachable");
        }
        me.reachability_changed(new_is_reachable, ctx)
    });
}

/// Reopens the last closed item (window or tab).
fn undo_close(_: &(), ctx: &mut AppContext) {
    UndoCloseStack::handle(ctx).update(ctx, |stack, ctx| {
        stack.undo_close(ctx);
    });
}
