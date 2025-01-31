use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use burn::{
    data::dataloader::{batcher::Batcher, DataLoader, DataLoaderBuilder},
    prelude::Backend,
    tensor::{Tensor, TensorData},
};
use burn_dataset::DataframeDataset;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct ZipItem {
    pub zip: u32,
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lng")]
    pub longitude: f64,
}

#[derive(Clone, Debug)]
pub struct ZipBatcher<B: Backend>(pub B::Device);

#[derive(Clone, Debug)]
pub struct ZipBatch<B: Backend> {
    pub zips: Tensor<B, 2>,
    pub locations: Tensor<B, 2>,
}

pub fn create_loader<B: Backend>(device: &B::Device) -> Arc<dyn DataLoader<ZipBatch<B>>> {
    let dataset = DataframeDataset::new(ziplocator_data::load_dataframe())
        .expect("Create dataset from dataframe");

    DataLoaderBuilder::new(ZipBatcher(device.clone()))
        .batch_size(100)
        .shuffle(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        )
        .build(dataset)
}

impl<B: Backend> Batcher<ZipItem, ZipBatch<B>> for ZipBatcher<B> {
    fn batch(&self, items: Vec<ZipItem>) -> ZipBatch<B> {
        let (zips, locations): (Vec<Tensor<B, 1>>, Vec<Tensor<B, 1>>) = items
            .into_iter()
            .map(|item| {
                let zip_data = TensorData::new::<f64, _>(vec![item.zip as f64], vec![1]);
                let location_data =
                    TensorData::new::<f64, _>(vec![item.latitude, item.longitude], vec![2]);
                (
                    Tensor::from_data(zip_data, &self.0),
                    Tensor::from_data(location_data, &self.0),
                )
            })
            .unzip();

        ZipBatch {
            zips: Tensor::stack(zips, 0).to_device(&self.0),
            locations: Tensor::stack(locations, 0).to_device(&self.0),
        }
    }
}
