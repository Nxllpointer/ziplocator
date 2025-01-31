use std::{io::Write, time::Duration};

use burn::{
    lr_scheduler::exponential::ExponentialLrSchedulerConfig,
    module::Module,
    optim::AdamConfig,
    prelude::Backend,
    record::{FullPrecisionSettings, PrettyJsonFileRecorder},
    tensor::backend::AutodiffBackend,
    train::{
        metric::{LearningRateMetric, LossMetric},
        LearnerBuilder, RegressionOutput, TrainOutput, TrainStep, ValidStep,
    },
};

impl<B: AutodiffBackend> TrainStep<crate::ZipBatch<B>, RegressionOutput<B>> for crate::ZipModel<B> {
    fn step(&self, item: crate::ZipBatch<B>) -> burn::train::TrainOutput<RegressionOutput<B>> {
        let output = self.forward_regression(item);
        TrainOutput::new(self, output.loss.backward(), output)
    }
}

impl<B: Backend> ValidStep<crate::ZipBatch<B>, RegressionOutput<B>> for crate::ZipModel<B> {
    fn step(&self, item: crate::ZipBatch<B>) -> RegressionOutput<B> {
        self.forward_regression(item)
    }
}

pub fn train<B: AutodiffBackend>(device: &B::Device) {
    let model = crate::ZipModel::<B>::new(device);
    let optimizer = AdamConfig::new().init();
    let lr_scheduler = ExponentialLrSchedulerConfig::new(0.01, 0.9998)
        .init()
        .unwrap();

    let loader_train = crate::create_loader(device);
    let loader_valid = crate::create_loader(device);

    let learner = LearnerBuilder::<B, _, _, _, _, _>::new(crate::ARTIFACT_DIR)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .metric_train(LearningRateMetric::new())
        .num_epochs(1000)
        .devices(vec![device.clone()])
        .build(model, optimizer, lr_scheduler);

    let model = learner.fit(loader_train, loader_valid);

    model
        .save_file(
            format!("{}{}", crate::ARTIFACT_DIR, crate::MODEL_FILE),
            &PrettyJsonFileRecorder::<FullPrecisionSettings>::new(),
        )
        .expect("Unable to save model");

    println!("Model saved!");

    std::io::stdout().flush().ok();
    std::thread::sleep(Duration::from_millis(100));
}
