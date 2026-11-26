use settings::Setting;
use warpui::{
    elements::{Flex, ParentElement},
    Element, Entity, SingletonEntity, TypedActionView, View, ViewContext, ViewHandle,
};

use crate::{
    appearance::Appearance,
    report_if_error,
    settings_view::settings_page::render_dropdown_item,
    util::file::external_editor::{settings::EditorChoice, EditorSettings, SUPPORTED_EDITORS},
    view_components::{Dropdown, DropdownItem},
};

#[derive(Debug, Clone)]
pub enum ExternalEditorAction {
    SetEditor(EditorChoice),
}

pub struct ExternalEditorView {
    editor_dropdown: ViewHandle<Dropdown<ExternalEditorAction>>,
}

impl ExternalEditorView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let editor = *EditorSettings::as_ref(ctx).open_file_editor;
        let editor_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            Self::init_editor_dropdown(editor, &mut dropdown, ctx);
            dropdown
        });
        ctx.subscribe_to_model(&EditorSettings::handle(ctx), |me, settings, _, ctx| {
            let editor = *settings.as_ref(ctx).open_file_editor;
            me.editor_dropdown.update(ctx, |dropdown, ctx| {
                Self::init_editor_dropdown(editor, dropdown, ctx);
            });
            ctx.notify();
        });
        Self { editor_dropdown }
    }

    fn init_editor_dropdown(
        editor: EditorChoice,
        dropdown: &mut Dropdown<ExternalEditorAction>,
        ctx: &mut ViewContext<Dropdown<ExternalEditorAction>>,
    ) {
        let mut items = vec![
            DropdownItem::new(
                "Default App",
                ExternalEditorAction::SetEditor(EditorChoice::SystemDefault),
            ),
            DropdownItem::new(
                "$EDITOR",
                ExternalEditorAction::SetEditor(EditorChoice::EnvEditor),
            ),
        ];
        for editor in SUPPORTED_EDITORS {
            if editor.is_installed(ctx) {
                items.push(DropdownItem::new(
                    format!("{editor}"),
                    ExternalEditorAction::SetEditor(EditorChoice::ExternalEditor(*editor)),
                ));
            }
        }
        dropdown.set_items(items, ctx);
        let selected = match editor {
            EditorChoice::SystemDefault => "Default App".to_owned(),
            EditorChoice::EnvEditor => "$EDITOR".to_owned(),
            EditorChoice::ExternalEditor(editor) => format!("{editor}"),
        };
        dropdown.set_selected_by_name(selected, ctx);
    }
}

impl Entity for ExternalEditorView {
    type Event = ();
}

impl View for ExternalEditorView {
    fn ui_name() -> &'static str {
        "ExternalEditorView"
    }

    fn render(&self, ctx: &warpui::AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(ctx);
        Flex::column()
            .with_child(render_dropdown_item::<ExternalEditorAction>(
                appearance,
                "Open files with",
                None,
                None,
                None,
                &self.editor_dropdown,
            ))
            .finish()
    }
}

impl TypedActionView for ExternalEditorView {
    type Action = ExternalEditorAction;
    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            ExternalEditorAction::SetEditor(editor) => {
                EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.open_file_editor.set_value(*editor, ctx));
                });
            }
        }
    }
}
