use iced::{
    futures::{SinkExt, Stream},
    widget::{self, image::Handle as ImageHandle},
    Element, Subscription,
};
use tokio::sync::mpsc;

use crate::map::{MapCommand, MapWorker};

#[derive(Default)]
pub struct State {
    map_controller: Option<mpsc::Sender<MapCommand>>,
    map_frame: Option<ImageHandle>,
}

#[derive(Debug)]
pub enum Message {
    SetMapController(mpsc::Sender<MapCommand>),
    UpdateMapFrame(ImageHandle),
}

fn view(state: &State) -> Element<Message> {
    if let Some(map_image) = &state.map_frame {
        widget::image(map_image).into()
    } else {
        widget::text("Hello world!").into()
    }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::SetMapController(controller) => state.map_controller = Some(controller),
        Message::UpdateMapFrame(handle) => state.map_frame = Some(handle),
    }
}

fn map_worker() -> impl Stream<Item = Message> {
    iced::stream::channel(10, |mut messages| async move {
        let (command_tx, command_rx) = mpsc::channel(10);
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
