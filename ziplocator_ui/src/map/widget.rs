use iced::{
    advanced::{layout::Node, widget::tree, Widget},
    widget::image::Handle as ImageHandle,
    Element, Length, Point, Rectangle, Size,
};
use tokio::sync::{mpsc, oneshot};

use super::worker::MapCommand;

pub struct MapWidget<'a, Message> {
    pub frame: &'a ImageHandle,
    pub controller: &'a mpsc::Sender<MapCommand>,
    pub location_clicked: &'a dyn Fn(f64, f64) -> Message,
}

#[derive(Default)]
struct MapState {
    grab_start: Option<Point>,
    location_rx: Option<oneshot::Receiver<(f64, f64)>>,
}

impl<'a, Message, Theme, Renderer: iced::advanced::image::Renderer<Handle = ImageHandle>>
    Widget<Message, Theme, Renderer> for MapWidget<'a, Message>
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
        let size = limits.max();
        self.controller
            .blocking_send(MapCommand::SetSize(size))
            .ok();
        Node::new(size)
    }

    fn update(
        &mut self,
        state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
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
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) => {
                    if let Some(screen_pos) = cursor.position_in(layout.bounds()) {
                        let (location_tx, location_rx) = oneshot::channel();
                        self.controller
                            .try_send(MapCommand::QueryLocation {
                                screen_pos,
                                location_tx,
                            })
                            .ok();
                        state.location_rx = Some(location_rx)
                    }
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

        if let Some(location_rx) = &mut state.location_rx {
            if let Ok((lat, lon)) = location_rx.try_recv() {
                shell.publish((self.location_clicked)(lat, lon));
            }
        }
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

impl<'a, Message: 'a, Theme, Renderer: iced::advanced::image::Renderer<Handle = ImageHandle>>
    From<MapWidget<'a, Message>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: MapWidget<'a, Message>) -> Self {
        Element::new(value)
    }
}
