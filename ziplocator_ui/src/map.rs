use galileo::{
    galileo_types::{cartesian::Size, latlon},
    render::WgpuRenderer,
    tile_scheme::TileIndex,
    DummyMessenger, Map, MapBuilder, MapView, TileSchema,
};
use tokio::sync::mpsc;

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

        let view = MapView::new(&latlon!(52.0, 0.0), tile_schema.lod_resolution(17).unwrap())
            .with_size(Size::new(512.0, 512.0));

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

    pub async fn run(self) {
        let renderer = WgpuRenderer::new_with_texture_rt(Size::new(512, 512))
            .await
            .expect("failed to create renderer");
        loop {
            self.map.load_layers();

            renderer
                .render(&self.map)
                .expect("failed to render the map");

            let bitmap = renderer
                .get_image()
                .await
                .expect("failed to get image bitmap from texture");

            self.frame_tx
                .send(iced::advanced::image::Handle::from_rgba(512, 512, bitmap))
                .await
                .ok();
        }
    }
}

pub enum MapCommand {}
