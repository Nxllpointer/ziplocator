use data_downloader::{DownloadRequest, InZipDownloadRequest};
use hex_literal::hex;
use polars::{frame::DataFrame, io::SerReader, prelude::CsvReadOptions};
use std::io::Cursor;

const DATASET_URL: &str =
    "https://simplemaps.com/static/data/us-zips/1.90/basic/simplemaps_uszips_basicv1.90.zip";

pub fn load_dataframe() -> DataFrame {
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
