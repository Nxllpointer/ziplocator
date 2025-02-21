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
    let dataframe = ziplocator_data::Dataset::load().dataframe();
    let dataset = DataframeDataset::new(dataframe).expect("Create dataset from dataframe");

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

pub fn create_zip_tensor<B: Backend>(device: &B::Device, zip: u32) -> Tensor<B, 2> {
    let digits: Vec<f64> = format!("{:b}", zip)
        .chars()
        .map(|d| d.to_digit(2).unwrap() as f64)
        .collect();
    let digits = std::iter::repeat(0.0)
        .take(crate::INPUT_SIZE - digits.len())
        .chain(digits)
        .collect();

    let zip_data = TensorData::new::<f64, _>(digits, vec![1, crate::INPUT_SIZE]);

    Tensor::from_data(zip_data, device)
}

impl<B: Backend> Batcher<ZipItem, ZipBatch<B>> for ZipBatcher<B> {
    fn batch(&self, items: Vec<ZipItem>) -> ZipBatch<B> {
        let (zips, locations): (Vec<Tensor<B, 2>>, Vec<Tensor<B, 2>>) = items
            .into_iter()
            .map(|item| {
                let location_data =
                    TensorData::new::<f64, _>(vec![item.latitude, item.longitude], vec![1, 2]);
                (
                    create_zip_tensor(&self.0, item.zip),
                    Tensor::from_data(location_data, &self.0),
                )
            })
            .unzip();

        ZipBatch {
            zips: Tensor::cat(zips, 0).to_device(&self.0),
            locations: Tensor::cat(locations, 0).to_device(&self.0),
        }
    }
}
