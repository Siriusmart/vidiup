use serde::{Deserialize, Serialize};

use crate::SavedFile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MasterConfig {
    pub hot_per_region: u32,
    pub timeout: u32,
    pub reverse_proxy: bool,
    pub port: u16,
}

impl SavedFile for MasterConfig {
    const PATH: &'static str = ".config/vidiup/master.json";
}
