mod map;

use iced::{
    futures::{SinkExt, Stream},
    widget::{self, image::Handle as ImageHandle},
    Element, Subscription,
};
use map::widget::MapWidget;
use tokio::sync::mpsc;

use crate::map::worker::{MapCommand, MapWorker};

#[derive(Default)]
pub struct State {
    map_controller: Option<mpsc::Sender<MapCommand>>,
    map_frame: Option<ImageHandle>,
    zip_code: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    SetMapController(mpsc::Sender<MapCommand>),
    UpdateMapFrame(ImageHandle),
    ZipCodeChanged(String),
    RunPrediction,
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
            MapWidget { controller, frame }.into()
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
        Message::RunPrediction => todo!(),
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
