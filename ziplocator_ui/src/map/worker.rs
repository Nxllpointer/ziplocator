use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use galileo::{
    galileo_types::{
        cartesian::{Point2d, Size},
        geo::{impls::GeoPoint2d, Crs, GeoPoint},
        geometry_type::GeoSpace2d,
        latlon,
    },
    layer::{data_provider::UrlImageProvider, FeatureLayer, RasterTileLayer},
    render::WgpuRenderer,
    symbol::CirclePointSymbol,
    tile_scheme::TileIndex,
    Color, Map, MapView, Messenger, TileSchema,
};
use iced::wgpu;
use tokio::sync::{mpsc, oneshot};

pub enum MapCommand {
    SetSize(iced::Size),
    Zoom {
        multiplier: f64,
    },
    Move {
        from: iced::Point,
        to: iced::Point,
    },
    PlacePins {
        prediction: Option<GeoPoint2d>,
        dataset: Option<GeoPoint2d>,
    },
    QueryLocation {
        screen_pos: iced::Point,
        location_tx: oneshot::Sender<(f64, f64)>,
    },
}

pub struct MapWorker {
    command_rx: mpsc::Receiver<MapCommand>,
    frame_tx: mpsc::Sender<iced::advanced::image::Handle>,
    redraw_requested: Arc<AtomicBool>,
    map: Map,
}

impl MapWorker {
    pub fn new(
        command_rx: mpsc::Receiver<MapCommand>,
        frame_tx: mpsc::Sender<iced::advanced::image::Handle>,
    ) -> Self {
        let tile_schema = TileSchema::web(18);
        let view = MapView::new(
            &latlon!(35.0, -95.0),
            tile_schema.lod_resolution(4).unwrap(),
        );

        let redraw_requested = Arc::new(AtomicBool::new(true));
        let redraw_messenger = RedrawMessenger(redraw_requested.clone());

        let tile_source = |index: &TileIndex| {
            format!(
                "https://tile.openstreetmap.org/{}/{}/{}.png",
                index.z, index.x, index.y
            )
        };
        let tile_provider = UrlImageProvider::new(tile_source);
        let raster = RasterTileLayer::new(
            tile_schema,
            tile_provider,
            Some(Arc::new(redraw_messenger.clone())),
        );

        let map = galileo::Map::new(
            view,
            vec![Box::new(raster)],
            Some(RedrawMessenger(redraw_requested.clone())),
        );

        Self {
            command_rx,
            frame_tx,
            redraw_requested,
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

        while !self.frame_tx.is_closed() {
            while let Ok(command) = self.command_rx.try_recv() {
                let view = self.map.target_view();
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
                    MapCommand::PlacePins {
                        prediction,
                        dataset,
                    } => {
                        let layers = self.map.layers_mut();
                        layers.truncate(1);

                        if let Some(dataset) = dataset {
                            layers.push(create_pin_layer(dataset, Color::BLUE));
                        }
                        if let Some(prediction) = prediction {
                            layers.push(create_pin_layer(prediction, Color::RED));
                        }

                        self.map.redraw();
                    }
                    MapCommand::QueryLocation {
                        screen_pos,
                        location_tx,
                    } => {
                        let screen_pos = [screen_pos.x as f64, screen_pos.y as f64].into();
                        if let Some(geo) = view.screen_to_map_geo(screen_pos) {
                            location_tx.send((geo.lat(), geo.lon())).ok();
                        }
                    }
                }
            }

            self.map.animate();

            if self.redraw_requested.swap(false, Ordering::SeqCst) {
                self.map.load_layers();
                renderer
                    .render(&self.map)
                    .expect("Failed to render the map");

                let bitmap = renderer
                    .get_image()
                    .await
                    .expect("Failed to get image bitmap from texture");

                let size = self.map.target_view().size();
                self.frame_tx
                    .send(iced::advanced::image::Handle::from_rgba(
                        size.width() as u32,
                        size.height() as u32,
                        bitmap,
                    ))
                    .await
                    .ok();
            }

            tokio::task::yield_now().await;
        }
    }
}

fn align_length(length: f32) -> u32 {
    (length / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as f32).ceil() as u32
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

fn create_pin_layer(
    location: GeoPoint2d,
    color: Color,
) -> FeatureLayer<GeoPoint2d, GeoPoint2d, CirclePointSymbol, GeoSpace2d> {
    FeatureLayer::new(
        vec![location],
        CirclePointSymbol::new(color, 10.0),
        Crs::WGS84,
    )
}

#[derive(Clone)]
struct RedrawMessenger(Arc<AtomicBool>);
impl Messenger for RedrawMessenger {
    fn request_redraw(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
