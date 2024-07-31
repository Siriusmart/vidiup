use serde::{Deserialize, Serialize};

use crate::SavedFile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceConfig {
    pub regions_selector: Vec<RegionSelectorEntry>,
    pub latency_thresholds: LatencyThresholds,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegionSelectorEntry {
    pub display: String,
    pub internal: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LatencyThresholds {
    pub good: u32,
    pub moderate: u32,
}

impl LatencyThresholds {
    pub fn quality(&self, latency: u32) -> &'static str {
        if latency < self.good {
            "good"
        } else if latency < self.moderate {
            "moderate"
        } else {
            "bad"
        }
    }
}

impl SavedFile for InterfaceConfig {
    const PATH: &'static str = ".config/vidiup/interface.json";
}
