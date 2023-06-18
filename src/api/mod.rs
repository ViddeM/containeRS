use serde::{Deserialize, Serialize};

pub mod container_spec;
pub mod images;

pub const INTERNAL_SERVER_ERROR: &str = "INTERNAL_SERVER_ERROR";

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    error: String,
}
