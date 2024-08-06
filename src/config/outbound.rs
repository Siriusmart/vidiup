use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SavedFile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutboundConfig {
    pub polling: PollingConfig,
    pub offsets: OffsetsConfig,
    pub poll_probabilities: PollProbabilitiesConfig,
    pub check_interval: u64,
}

impl SavedFile for OutboundConfig {
    const PATH: &'static str = ".config/vidiup/outbound.json";
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollProbabilitiesConfig {
    pub hot: f32,
    pub recovered: f32,
    pub recovering: f32,
    pub dead: f32,
    pub stashed_recovering: f32,
    pub stashed_dead: f32,
    pub stashed: f32,
    pub pending: f32,
}

impl PollProbabilitiesConfig {
    pub fn total(&self) -> f32 {
        self.hot
            + self.recovered
            + self.recovering
            + self.dead
            + self.stashed_recovering
            + self.stashed_dead
            + self.stashed
            + self.pending
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollingConfig {
    pub interval: u64,
    pub features: PollingFeaturesConfig,
    pub max_concurrent: u32,
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
