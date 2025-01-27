pub mod data;
pub mod model;
mod train;

pub use data::*;
pub use model::*;
pub use train::train;

pub const ARTIFACT_DIR: &str = "./learn/";
pub const MODEL_FILE: &str = "model.json";
