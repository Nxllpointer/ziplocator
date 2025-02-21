use burn::{
    module::Module,
    nn::{loss::MseLoss, Linear, LinearConfig, Relu, Tanh},
    prelude::Backend,
    tensor::Tensor,
    train::RegressionOutput,
};

pub const INPUT_SIZE: usize = 18;

#[derive(Module, Debug)]
pub struct ZipModel<B: Backend> {
    lin1: Linear<B>,
    lin2: Linear<B>,
    lin3: Linear<B>,
    lin4: Linear<B>,
    lin5: Linear<B>,
}

impl<B: Backend> ZipModel<B> {
    pub fn new(device: &B::Device) -> Self {
        Self {
            lin1: LinearConfig::new(INPUT_SIZE, 64).init(device),
            lin2: LinearConfig::new(64, 32).init(device),
            lin3: LinearConfig::new(32, 16).init(device),
            lin4: LinearConfig::new(16, 8).init(device),
            lin5: LinearConfig::new(8, 2).init(device),
        }
    }

    pub fn forward(
        &self,
        mut x: Tensor<B, 2>,
        recorder: Option<&mut LayerOutputRecorder>,
    ) -> Tensor<B, 2> {
        let mut record: Box<dyn FnMut(&Tensor<B, 2>)> = if let Some(recorder) = recorder {
            Box::new(|tensor| {
                recorder
                    .layers
                    .push(tensor.to_data().convert::<f64>().into_vec().unwrap());
            })
        } else {
            Box::new(|_| {})
        };

        record(&x);

        x = self.lin1.forward(x);
        x = Relu.forward(x);
        record(&x);

        x = self.lin2.forward(x);
        x = Tanh.forward(x);
        record(&x);

        x = self.lin3.forward(x);
        x = Tanh.forward(x);
        record(&x);

        x = self.lin4.forward(x);
        x = Tanh.forward(x);
        record(&x);

        x = self.lin5.forward(x);
        record(&x);

        x = x * 100.0;
        record(&x);

        x
    }

    pub fn forward_regression(&self, batch: crate::ZipBatch<B>) -> RegressionOutput<B> {
        let outputs = self.forward(batch.zips.clone(), None);
        let loss = MseLoss::new()
            .forward_no_reduction(outputs.clone(), batch.locations.clone())
            .sum_dim(1)
            .sqrt()
            .squeeze(1);
        RegressionOutput::new(loss, outputs, batch.locations)
    }
}

#[derive(Default)]
pub struct LayerOutputRecorder {
    pub layers: Vec<Vec<f64>>,
}
