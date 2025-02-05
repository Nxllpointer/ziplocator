use std::io::BufRead;

use burn::backend::{ndarray::NdArrayDevice, NdArray};
use ziplocator_nn::Inferrer;

fn main() {
    let inferrer = ziplocator_nn::InferrerImpl::<NdArray>::load(NdArrayDevice::Cpu);
    println!("Enter zip code:");
    let zip: u32 = std::io::stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .parse()
        .expect("Invalid zip code");

    dbg!(inferrer.infer(zip, None));
}
