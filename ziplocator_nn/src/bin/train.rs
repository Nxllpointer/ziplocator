use burn::{
    backend::{ndarray::NdArrayDevice, NdArray},
    data::dataloader::batcher::Batcher,
};
use burn_dataset::Dataset;

fn main() {
    let device = NdArrayDevice::Cpu;
    let dataset = ziplocator_nn::data::load_dataset();
    let item1 = dataset.get(0).unwrap();
    let item2 = dataset.get(1).unwrap();
    let batcher = ziplocator_nn::data::ZipBatcher::<NdArray>(device.clone());
    let batch = batcher.batch(vec![item1, item2]);
    dbg!(batch);
}
