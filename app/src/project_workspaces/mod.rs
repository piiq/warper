use crate::{persistence::ModelEvent, report_if_error};
use anyhow::Context;
use chrono::Utc;
use itertools::Itertools;
use std::{collections::HashMap, path::PathBuf, sync::mpsc::SyncSender};
use warpui::{Entity, ModelContext, SingletonEntity};
pub mod metadata;
use metadata::WorkspaceMetadata;

/// Repository records used by tab-config repository and worktree pickers.
pub struct PersistedWorkspace {
    workspaces: HashMap<PathBuf, WorkspaceMetadata>,
    model_event_sender: Option<SyncSender<ModelEvent>>,
}

#[derive(Debug, Clone)]
pub enum PersistedWorkspaceEvent {
    WorkspaceAdded { path: PathBuf },
}
impl Entity for PersistedWorkspace {
    type Event = PersistedWorkspaceEvent;
}
impl SingletonEntity for PersistedWorkspace {}

impl PersistedWorkspace {
    #[cfg(test)]
    pub fn new_for_test(ctx: &mut ModelContext<Self>) -> Self {
        Self::new(vec![], None, ctx)
    }

    pub fn new(
        metadata: Vec<WorkspaceMetadata>,
        model_event_sender: Option<SyncSender<ModelEvent>>,
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Self {
            workspaces: metadata
                .into_iter()
                .map(|metadata| (metadata.path.clone(), metadata))
                .collect(),
            model_event_sender,
        }
    }

    pub fn user_added_workspace(&mut self, path: PathBuf, ctx: &mut ModelContext<Self>) {
        let metadata = self
            .workspaces
            .entry(path.clone())
            .or_insert_with(|| WorkspaceMetadata {
                path: path.clone(),
                ..Default::default()
            });
        metadata.navigated_ts = Some(Utc::now());
        if let Some(sender) = &self.model_event_sender {
            report_if_error!(sender
                .send(ModelEvent::UpsertWorkspaceMetadata {
                    metadata: Box::new(metadata.clone())
                })
                .context("Unable to save repository metadata to sqlite"));
        }
        ctx.emit(PersistedWorkspaceEvent::WorkspaceAdded { path });
    }

    pub fn workspaces(&self) -> impl Iterator<Item = WorkspaceMetadata> + '_ {
        self.workspaces
            .values()
            .cloned()
            .sorted_by(WorkspaceMetadata::most_recently_touched)
    }
}
