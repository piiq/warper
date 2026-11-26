use crate::search::{
    action::CommandBindingDataSource,
    binding_source::BindingSource,
    command_palette::{
        mixer::{CommandPaletteItemAction, ItemSummary},
        navigation,
        new_session::NewSessionDataSource,
        CommandPaletteMixer,
    },
    data_source::QueryResult,
    QueryFilter,
};
use crate::session_management::SessionSource;
use std::collections::HashSet;
use warp_core::features::FeatureFlag;
use warpui::{keymap::BindingId, AppContext, Entity, ModelContext, ModelHandle};

pub struct DataSourceStore {
    actions_data_source: ModelHandle<CommandBindingDataSource>,
    sessions_data_source: ModelHandle<navigation::DataSource>,
    new_session_data_source: Option<ModelHandle<NewSessionDataSource>>,
}
impl DataSourceStore {
    pub fn new(
        binding_source: ModelHandle<BindingSource>,
        active_session_handle: ModelHandle<SessionSource>,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        let actions_data_source =
            ctx.add_model(|ctx| CommandBindingDataSource::new(binding_source.clone(), ctx));
        let sessions_data_source =
            ctx.add_model(|_| navigation::DataSource::new(active_session_handle));
        let new_session_data_source = (FeatureFlag::ShellSelector.is_enabled()
            && cfg!(feature = "local_tty"))
        .then(|| ctx.add_model(|ctx| NewSessionDataSource::new(binding_source, ctx)));
        Self {
            actions_data_source,
            sessions_data_source,
            new_session_data_source,
        }
    }
    pub fn reset_search_mixer(
        &mut self,
        mixer: ModelHandle<CommandPaletteMixer>,
        ctx: &mut ModelContext<Self>,
    ) {
        mixer.update(ctx, |mixer, ctx| {
            mixer.reset(ctx);
            mixer.add_sync_source(
                self.sessions_data_source.clone(),
                HashSet::from([QueryFilter::Sessions]),
            );
            mixer.add_sync_source(
                self.actions_data_source.clone(),
                HashSet::from([QueryFilter::Actions]),
            );
            if let Some(source) = &self.new_session_data_source {
                mixer.add_sync_source(source.clone(), HashSet::from([QueryFilter::Actions]));
            }
            ctx.notify();
        });
    }
    pub fn query_result_from_summary(
        &self,
        summary: &ItemSummary,
        app: &AppContext,
    ) -> Option<QueryResult<CommandPaletteItemAction>> {
        match summary {
            ItemSummary::Action { binding_id } => self
                .actions_data_source
                .as_ref(app)
                .query_result(*binding_id),
            ItemSummary::Session { pane_view_locator } => self
                .sessions_data_source
                .as_ref(app)
                .query_result(*pane_view_locator, app),
            ItemSummary::NewSession { id } => self
                .new_session_data_source
                .as_ref()
                .and_then(|source| source.as_ref(app).query_result(id)),
            ItemSummary::NoOp => None,
        }
    }
    pub fn query_result_for_binding_id(
        &self,
        binding_id: BindingId,
        app: &AppContext,
    ) -> Option<QueryResult<CommandPaletteItemAction>> {
        self.query_result_from_summary(&ItemSummary::Action { binding_id }, app)
    }
}
impl Entity for DataSourceStore {
    type Event = ();
}
