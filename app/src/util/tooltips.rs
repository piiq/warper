//! Shared tooltip UI components for file path and link tooltips

use warpui::{
    elements::{Border, Container, CornerRadius, Flex, MouseStateHandle, ParentElement, Radius},
    ui_components::components::{Coords, UiComponent, UiComponentStyles},
    Element, EventContext,
};

use crate::appearance::Appearance;

/// A link to be shown in a tooltip
pub struct TooltipLink<OnClick> {
    pub text: String,
    pub on_click: OnClick,
    /// Optional detail text to show after the link (e.g., "[Cmd Click]")
    pub detail: Option<String>,
    pub mouse_state: MouseStateHandle,
}

impl<OnClick> TooltipLink<OnClick> {
    pub fn new(text: String, on_click: OnClick, mouse_state: MouseStateHandle) -> Self {
        Self {
            text,
            on_click,
            detail: None,
            mouse_state,
        }
    }

    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }
}

/// Render a tooltip with one or more links.
///
/// This is generic over the click handler type to support different action dispatch mechanisms.
pub fn render_tooltip<OnClick>(
    tooltip_links: impl IntoIterator<Item = TooltipLink<OnClick>>,
    appearance: &Appearance,
) -> Box<dyn Element>
where
    OnClick: 'static + Fn(&mut EventContext),
{
    let mut tooltip = Flex::column();
    let mut links = Vec::new();
    let mut first = true;
    let background_color = appearance.theme().tooltip_background();

    for link in tooltip_links.into_iter() {
        if !first {
            links.push(
                Container::new(
                    appearance
                        .ui_builder()
                        .span(" | ".to_string())
                        .build()
                        .finish(),
                )
                .with_horizontal_padding(8.)
                .finish(),
            );
        }

        let on_click = link.on_click;
        links.push(
            appearance
                .ui_builder()
                .tooltip_link(
                    link.text,
                    None,
                    Some(Box::new(move |ctx| {
                        on_click(ctx);
                    })),
                    link.mouse_state,
                )
                .soft_wrap(false)
                .build()
                .finish(),
        );

        if let Some(detail) = link.detail {
            links.push(
                appearance
                    .ui_builder()
                    .span(detail)
                    .with_style(UiComponentStyles {
                        margin: Some(Coords::default().left(4.)),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            );
        }

        first = false;
    }

    let link_row = if links.is_empty() {
        None
    } else {
        Some(Flex::row().with_children(links).finish())
    };

    if let Some(link_row) = link_row {
        tooltip.add_child(link_row);
    }
    let tooltip_element = Container::new(tooltip.finish())
        .with_vertical_padding(4.)
        .with_horizontal_padding(6.)
        .finish();

    Container::new(tooltip_element)
        .with_background(background_color)
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
        .with_border(Border::all(1.).with_border_fill(appearance.theme().outline()))
        .finish()
}
