use std::io::Cursor;

use burn::{
    data::dataloader::batcher::Batcher,
    prelude::Backend,
    tensor::{Tensor, TensorData},
};
use burn_dataset::DataframeDataset;
use data_downloader::{DownloadRequest, InZipDownloadRequest};
use hex_literal::hex;
use polars::{frame::DataFrame, io::SerReader, prelude::CsvReadOptions};
use serde::Deserialize;

const DATASET_URL: &str =
    "https://simplemaps.com/static/data/us-zips/1.90/basic/simplemaps_uszips_basicv1.90.zip";

#[derive(Deserialize, Clone, Debug)]
pub struct ZipItem {
    zip: u32,
    #[serde(rename = "lat")]
    latitude: f64,
    #[serde(rename = "lng")]
    longitude: f64,
}

pub type ZipDataset = DataframeDataset<ZipItem>;

#[derive(Debug)]
pub struct ZipBatcher<B: Backend>(pub B::Device);

#[derive(Debug)]
pub struct ZipBatch<B: Backend> {
    zips: Tensor<B, 2>,
    locations: Tensor<B, 2>,
}

fn load_dataframe() -> DataFrame {
    let dataset = data_downloader::get(&InZipDownloadRequest {
        parent: &DownloadRequest {
            url: DATASET_URL,
            sha256_hash: &hex!("911765CB2433F7BDFF22D2817CA1B96BDE8F4B6F5C10FB9AEA1F3310DC04F1F8"),
        },
        path: "uszips.csv",
        sha256_hash: &hex!("0B8F9D378D8868F42324788A457A17434E38BB364060055D5C338A2FFE512285"),
    })
    .expect("Downloading dataset");

    let dataset = Cursor::new(dataset);

    CsvReadOptions::default()
        .with_has_header(true)
        .into_reader_with_file_handle(dataset)
        .finish()
        .expect("Create dataframe")
        .select(["zip", "lat", "lng"])
        .expect("Selecting columns")
}

pub fn load_dataset() -> ZipDataset {
    DataframeDataset::new(load_dataframe()).expect("Create dataset from dataframe")
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
