use burn::backend::{ndarray::NdArrayDevice, Autodiff, NdArray};

fn main() {
    ziplocator_nn::train::<Autodiff<NdArray>>(&NdArrayDevice::Cpu);
}
