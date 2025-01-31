mod map;

use galileo::galileo_types::latlon;
use iced::{
    futures::{SinkExt, Stream},
    widget::{self, image::Handle as ImageHandle},
    Color, Element, Length, Subscription,
};
use map::widget::MapWidget;
use tokio::sync::mpsc;

use crate::map::worker::{MapCommand, MapWorker};

#[derive(Default)]
pub struct State {
    inferrer: Box<dyn ziplocator_nn::Inferrer>,
    map_controller: Option<mpsc::Sender<MapCommand>>,
    map_frame: Option<ImageHandle>,
    zip_code: String,
    legend_visible: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    SetMapController(mpsc::Sender<MapCommand>),
    UpdateMapFrame(ImageHandle),
    ZipCodeChanged(String),
    RunPrediction,
    OpenLink(String),
}

fn view(state: &State) -> Element<Message> {
    let controls = widget::container(
        widget::row![
            widget::text_input("Enter zip code...", &state.zip_code)
                .style(|theme, status| {
                    let mut style = widget::text_input::default(theme, status);
                    if let Err(_) = state.zip_code.parse::<u32>() {
                        style.border.color = theme.palette().danger;
                    }
                    style
                })
                .on_input(Message::ZipCodeChanged)
                .on_submit(Message::RunPrediction)
                .width(200),
            widget::button(widget::text!("Predict!"))
                .style(widget::button::primary)
                .on_press(Message::RunPrediction)
        ]
        .spacing(10),
    )
    .padding(10);

    let map: Element<_> =
        if let (Some(controller), Some(frame)) = (&state.map_controller, &state.map_frame) {
            let map = MapWidget { controller, frame };

            let legend = if state.legend_visible {
                Some(
                    widget::right(
                        widget::container(widget::column![
                            widget::text!("ʘ Prediction").color(iced::color!(0xFF0000)),
                        ])
                        .style(|theme: &iced::Theme| widget::container::Style {
                            background: Some(theme.palette().background.into()),
                            ..widget::container::rounded_box(theme)
                        })
                        .padding(10),
                    )
                    .padding(5),
                )
            } else {
                None
            };

            let attribution = widget::container(
                widget::button(widget::text!("Data from OpenStreetMap"))
                    .style(|theme, status| widget::button::Style {
                        text_color: Color::BLACK,
                        ..widget::button::text(theme, status)
                    })
                    .on_press(Message::OpenLink(
                        "https://www.openstreetmap.org/fixthemap".into(),
                    )),
            )
            .align_right(Length::Fill)
            .align_bottom(Length::Fill);

            widget::Stack::new()
                .push(map)
                .push_maybe(legend)
                .push(attribution)
                .into()
        } else {
            widget::row![].into()
        };

    widget::column![controls, map].into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::SetMapController(controller) => state.map_controller = Some(controller),
        Message::UpdateMapFrame(handle) => state.map_frame = Some(handle),
        Message::ZipCodeChanged(zip_code) => state.zip_code = zip_code,
        Message::RunPrediction => {
            let (Some(map_controller), Ok(zip)) = (&state.map_controller, state.zip_code.parse())
            else {
                return;
            };

            let prediction = state.inferrer.infer(zip);

            map_controller
                .try_send(MapCommand::PlacePins {
                    prediction: Some(latlon!(prediction.latitude, prediction.longitude)),
                    dataset: None,
                })
                .ok();

            state.legend_visible = true;
        }
        Message::OpenLink(link) => {
            opener::open(link).ok();
        }
    }
}

fn map_worker() -> impl Stream<Item = Message> {
    iced::stream::channel(10, |mut messages| async move {
        let (command_tx, command_rx) = mpsc::channel(999);
        let (frame_tx, mut frame_rx) = mpsc::channel(10);

        messages
            .send(Message::SetMapController(command_tx))
            .await
            .ok();

        tokio::spawn(MapWorker::new(command_rx, frame_tx).run());

        while let Some(frame) = frame_rx.recv().await {
            messages.send(Message::UpdateMapFrame(frame)).await.ok();
        }
    })
}

pub fn run() {
    iced::application("Ziplocator UI", update, view)
        .subscription(|_| Subscription::run(map_worker))
        .executor::<tokio::runtime::Runtime>()
        .run()
        .unwrap();
}
