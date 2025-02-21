use burn::{
    backend::{ndarray::NdArrayDevice, NdArray},
    module::Module,
    prelude::Backend,
    record::{FullPrecisionSettings, PrettyJsonFileRecorder},
};

pub trait Inferrer {
    fn infer(&self, zip: u32, recorder: Option<&mut crate::LayerOutputRecorder>) -> crate::ZipItem;
}

pub struct InferrerImpl<B: Backend> {
    device: B::Device,
    model: crate::ZipModel<B>,
}

impl<B: Backend> InferrerImpl<B> {
    pub fn load(device: B::Device) -> Self {
        let model = crate::ZipModel::<B>::new(&device)
            .load_file(
                format!("{}{}", crate::ARTIFACT_DIR, crate::MODEL_FILE),
                &PrettyJsonFileRecorder::<FullPrecisionSettings>::new(),
                &device,
            )
            .expect("Unable to load model from file");

        Self { device, model }
    }
}

impl<B: Backend> Inferrer for InferrerImpl<B> {
    fn infer(&self, zip: u32, recorder: Option<&mut crate::LayerOutputRecorder>) -> crate::ZipItem {
        let zips = crate::create_zip_tensor(&self.device, zip);

        let locations = self.model.forward(zips, recorder);
        let locations_data = locations.into_data().to_vec::<f32>().unwrap();

        crate::ZipItem {
            zip,
            latitude: locations_data[0] as f64,
            longitude: locations_data[1] as f64,
        }
    }
}

impl Default for Box<dyn Inferrer> {
    fn default() -> Self {
        Box::new(InferrerImpl::<NdArray>::load(NdArrayDevice::Cpu))
    }
}
