use std::time::Duration;

use galileo::{
    galileo_types::{
        cartesian::{Point2d, Size},
        latlon,
    },
    render::WgpuRenderer,
    tile_scheme::TileIndex,
    DummyMessenger, Map, MapBuilder, MapView, TileSchema,
};
use iced::wgpu;
use tokio::sync::mpsc;

pub enum MapCommand {
    SetSize(iced::Size),
    Zoom { multiplier: f64 },
    Move { from: iced::Point, to: iced::Point },
}

pub struct MapWorker {
    command_rx: mpsc::Receiver<MapCommand>,
    frame_tx: mpsc::Sender<iced::advanced::image::Handle>,
    map: Map,
}

impl MapWorker {
    pub fn new(
        command_rx: mpsc::Receiver<MapCommand>,
        frame_tx: mpsc::Sender<iced::advanced::image::Handle>,
    ) -> Self {
        let tile_schema = TileSchema::web(18);

        let view = MapView::new(&latlon!(52.0, 0.0), tile_schema.lod_resolution(17).unwrap());

        let raster = MapBuilder::create_raster_tile_layer(
            |index: &TileIndex| {
                format!(
                    "https://tile.openstreetmap.org/{}/{}/{}.png",
                    index.z, index.x, index.y
                )
            },
            tile_schema,
        );

        let map = galileo::Map::new(view, vec![Box::new(raster)], Some(DummyMessenger {}));

        Self {
            command_rx,
            frame_tx,
            map,
        }
    }

    pub async fn run(mut self) {
        let mut renderer = WgpuRenderer::new_with_texture_rt(Size::new(
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
        ))
        .await
        .expect("failed to create renderer");

        loop {
            while let Ok(command) = self.command_rx.try_recv() {
                let view = self.map.view();
                match command {
                    MapCommand::SetSize(size) => {
                        let new_size =
                            Size::new(align_length(size.width), align_length(size.height));
                        if new_size != view.size().cast() {
                            renderer.resize(new_size);
                            self.map
                                .animate_to(view.with_size(new_size.cast()), Duration::ZERO);
                        }
                    }
                    MapCommand::Zoom { multiplier } => {
                        self.map.animate_to(
                            view.with_resolution(view.resolution() * multiplier),
                            Duration::ZERO,
                        );
                    }
                    MapCommand::Move { from, to } => {
                        let from = Point2d::new(from.x as f64, from.y as f64);
                        let to = Point2d::new(to.x as f64, to.y as f64);
                        self.map
                            .animate_to(view.translate_by_pixels(from, to), Duration::ZERO);
                    }
                }
            }

            self.map.animate();
            self.map.load_layers();
            renderer
                .render(&self.map)
                .expect("failed to render the map");

            let bitmap = renderer
                .get_image()
                .await
                .expect("failed to get image bitmap from texture");

            let size = self.map.view().size();
            self.frame_tx
                .send(iced::advanced::image::Handle::from_rgba(
                    size.width() as u32,
                    size.height() as u32,
                    bitmap,
                ))
                .await
                .ok();
        }
    }
}

fn align_length(length: f32) -> u32 {
    (length / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as f32).ceil() as u32
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}
