use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct CurrentClientInfo {
    pub(crate) local_ip: String,
    pub(crate) port: u16,
    pub(crate) name: String,
    pub(crate) save_path: PathBuf,
}