pub mod data;
mod infer;
pub mod model;
mod train;

pub use data::*;
pub use infer::*;
pub use model::*;
pub use train::*;

pub const ARTIFACT_DIR: &str = "./learn/";
pub const MODEL_FILE: &str = "model.json";
