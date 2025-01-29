use iced::{
    advanced::{layout::Node, widget::tree, Widget},
    widget::image::Handle as ImageHandle,
    Element, Length, Point, Rectangle, Size,
};
use tokio::sync::mpsc;

use super::worker::MapCommand;

pub struct MapWidget<'a> {
    pub frame: &'a ImageHandle,
    pub controller: &'a mpsc::Sender<MapCommand>,
}

#[derive(Default)]
struct MapState {
    grab_start: Option<Point>,
}

impl<'a, Message, Theme, Renderer: iced::advanced::image::Renderer<Handle = ImageHandle>>
    Widget<Message, Theme, Renderer> for MapWidget<'a>
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        tree::Tag::of::<MapState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        tree::State::new(MapState::default())
    }

    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        Node::new(limits.max())
    }

    fn update(
        &mut self,
        state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state: &mut MapState = state.state.downcast_mut();

        match event {
            iced::Event::Mouse(event) => match event {
                iced::mouse::Event::WheelScrolled { delta } => match delta {
                    iced::mouse::ScrollDelta::Lines { y, .. }
                    | iced::mouse::ScrollDelta::Pixels { y, .. } => {
                        self.controller
                            .try_send(MapCommand::Zoom {
                                multiplier: 1.0 - (y.signum() as f64 * 0.1),
                            })
                            .ok();
                    }
                },
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    state.grab_start = cursor.position_over(layout.bounds());
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    state.grab_start = None;
                }
                iced::mouse::Event::CursorMoved { position } => {
                    if let Some(start_pos) = state.grab_start {
                        self.controller
                            .try_send(MapCommand::Move {
                                from: start_pos,
                                to: position,
                            })
                            .ok();
                        state.grab_start = Some(position);
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        renderer.with_layer(bounds, |renderer| {
            let image_size = renderer.measure_image(&self.frame);
            let image_size = Size::new(image_size.width as f32, image_size.height as f32);
            let draw_bounds = Rectangle::new(bounds.position(), image_size);
            renderer.draw_image(self.frame.into(), draw_bounds);
        });
    }
}

impl<'a, Message, Theme, Renderer: iced::advanced::image::Renderer<Handle = ImageHandle>>
    From<MapWidget<'a>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: MapWidget<'a>) -> Self {
        Element::new(value)
    }
}
