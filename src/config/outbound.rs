use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SavedFile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutboundConfig {
    pub polling: PollingConfig,
    pub offsets: OffsetsConfig,
}

impl SavedFile for OutboundConfig {
    const PATH: &'static str = ".config/vidiup/outbound.json";
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollingConfig {
    pub interval: u64,
    pub features: PollingFeaturesConfig,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollingFeaturesConfig {
    pub video: bool,
    pub playlist: bool,
    pub search: bool,
    pub channel: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OffsetsConfig(pub HashMap<String, i32>);
