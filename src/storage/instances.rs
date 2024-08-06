use std::collections::HashMap;

use chrono::{DateTime, Utc};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct InstancesRecords(pub HashMap<String, RegionRecords>);

impl SavedFile for InstancesRecords {
    const PATH: &'static str = ".local/share/vidiup/instances.json";
}

impl InstancesRecords {
    pub fn stat(&self) -> (u32, u32, u32, u32) {
        let mut up = 0;
        let mut recovering = 0;
        let mut dead = 0;
        let mut pending = 0;

        for record in self.0.values() {
            let (a, b, c, d) = record.stat();
            up += a;
            recovering += b;
            dead += c;
            pending += d;
        }

        (up as u32, recovering as u32, dead as u32, pending as u32)
    }

    pub fn as_global(&mut self) -> RegionRecords {
        let mut out = RegionRecords::default();

        for region in self.0.values() {
            out.hot.append(&mut region.hot.clone());
            out.recovered.append(&mut region.recovered.clone());
            out.recovering.append(&mut region.recovering.clone());
            out.dead.append(&mut region.dead.clone());
            out.stashed_recovering
                .append(&mut region.stashed_recovering.clone());
            out.stashed_dead.append(&mut region.stashed_dead.clone());
            out.stashed.append(&mut region.stashed.clone());
            out.pending.append(&mut region.pending.clone());
        }

        out
    }

    pub fn update_single(&mut self, instance: &str, record: PolledSingleRecord) {
        for region in self.0.values_mut() {
            if region.update_single(instance, record.clone()) {
                return;
            }
        }
    }

    pub fn add(&mut self, instance: String, region: String, backer: String) {
        for (region_current, records) in self.0.iter_mut() {
            if records.add_backer(&instance, &backer) {
                if region != region_current.as_str() {
                    warn!("{instance} may be in {region} (currently in {region_current})");
                } else {
                    let record = self.clone();
                    tokio::spawn(async move {
                        let _ = record.save().await;
                    });
                }
                return;
            }
        }

        self.0
            .entry(region.clone())
            .or_insert(RegionRecords::default())
            .pending
            .push(InstanceRecord {
                address: instance,
                region,
                backer: vec![backer],
            });

        let record = self.clone();
        tokio::spawn(async move {
            let _ = record.save().await;
            INSTANCES_STATS.get().unwrap().lock().unwrap().3 += 1;
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RegionRecords {
    pub hot: Vec<HotRecord>,
    pub recovered: Vec<HotRecord>,
    pub recovering: Vec<HotRecord>,
    pub dead: Vec<(DeadRecord, f64)>,
    pub stashed_recovering: Vec<InstanceRecord>,
    pub stashed_dead: Vec<DeadRecord>,
    pub stashed: Vec<InstanceRecord>,
    pub pending: Vec<InstanceRecord>,
}

impl RegionRecords {
    pub fn add_backer(&mut self, instance: &str, ip: &str) -> bool {
        let ip = ip.to_string();
        if let Some(instance) = self
            .hot
            .iter_mut()
            .chain(self.recovered.iter_mut())
            .chain(self.recovering.iter_mut())
            .map(|entry| &mut entry.instance)
            .chain(self.dead.iter_mut().map(|entry| &mut entry.0.instance))
            .chain(
                self.stashed_dead
                    .iter_mut()
                    .map(|entry| &mut entry.instance),
            )
            .chain(
                self.stashed_recovering
                    .iter_mut()
                    .chain(self.stashed.iter_mut().chain(self.pending.iter_mut())),
            )
            .find(|entry| entry.address == instance)
        {
            if !instance.backer.contains(&ip) {
                instance.backer.push(ip)
            }

            true
        } else {
            false
        }
    }

    pub fn update_weight(&mut self, instance: &str, multiplier: f64) -> bool {
        if let Some(hotrecord) = self
            .hot
            .iter_mut()
            .find(|record| record.instance.address == instance)
        {
            hotrecord.update_weight(multiplier);
            true
        } else {
            false
        }
    }

    pub fn kill(&mut self, instance: &str) -> bool {
        if let Some(i) = self
            .hot
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            let removed = self.hot.remove(i);
            self.dead.push((
                DeadRecord {
                    instance: removed.instance,
                    dead_since: Utc::now(),
                },
                removed.weight,
            ));
            return true;
        }

        if let Some(i) = self
            .recovered
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            let removed = self.recovered.remove(i);
            self.dead.push((
                DeadRecord {
                    instance: removed.instance,
                    dead_since: Utc::now(),
                },
                removed.weight,
            ));
            return true;
        }

        if let Some(i) = self
            .recovering
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            let removed = self.recovering.remove(i);
            self.dead.push((
                DeadRecord {
                    instance: removed.instance,
                    dead_since: Utc::now(),
                },
                removed.weight,
            ));
            return true;
        }

        if let Some(i) = self
            .stashed_recovering
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed_dead.push(DeadRecord {
                instance: self.stashed_recovering.remove(i),
                dead_since: Utc::now(),
            });
            return true;
        }

        if let Some(i) = self
            .stashed
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed_dead.push(DeadRecord {
                instance: self.stashed.remove(i),
                dead_since: Utc::now(),
            });
            return true;
        }

        if let Some(i) = self
            .pending
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed_dead.push(DeadRecord {
                instance: self.pending.remove(i),
                dead_since: Utc::now(),
            });
            return true;
        }

        false
    }

    pub fn rest(&mut self, instance: &str) -> bool {
        if let Some(i) = self
            .hot
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            self.recovering.push(self.hot.remove(i));
            return true;
        }

        if let Some(i) = self
            .recovered
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            self.recovering.push(self.recovered.remove(i));
            return true;
        }

        if let Some(i) = self
            .dead
            .iter()
            .position(|hotrecord| hotrecord.0.instance.address == instance)
        {
            let removed = self.dead.remove(i);
            self.recovering.push(HotRecord {
                instance: removed.0.instance,
                weight: removed.1,
            });
            return true;
        }

        if let Some(i) = self
            .stashed
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed_recovering.push(self.stashed.remove(i));
            return true;
        }

        if let Some(i) = self
            .stashed_dead
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            self.stashed_recovering
                .push(self.stashed_dead.remove(i).instance);
            return true;
        }

        if let Some(i) = self
            .pending
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed_recovering.push(self.pending.remove(i));
            return true;
        }

        false
    }

    pub fn revive(&mut self, instance: &str) -> bool {
        if let Some(i) = self
            .dead
            .iter()
            .position(|hotrecord| hotrecord.0.instance.address == instance)
        {
            let removed = self.dead.remove(i);
            self.recovered.push(HotRecord {
                instance: removed.0.instance,
                weight: removed.1,
            });
            return true;
        }

        if let Some(i) = self
            .recovering
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            let removed = self.recovering.remove(i);
            self.recovered.push(removed);
            return true;
        }

        if let Some(i) = self
            .stashed_dead
            .iter()
            .position(|hotrecord| hotrecord.instance.address == instance)
        {
            self.stashed.push(self.stashed_dead.remove(i).instance);
            return true;
        }

        if let Some(i) = self
            .stashed_recovering
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed.push(self.stashed_recovering.remove(i));
            return true;
        }

        if let Some(i) = self
            .pending
            .iter()
            .position(|hotrecord| hotrecord.address == instance)
        {
            self.stashed.push(self.pending.remove(i));
            return true;
        }

        false
    }

    pub fn stat(&self) -> (usize, usize, usize, usize) {
        (
            self.hot.len() + self.recovered.len() + self.stashed.len(),
            self.recovering.len() + self.stashed_recovering.len(),
            self.dead.len() + self.stashed_dead.len(),
            self.pending.len(),
        )
    }

    pub fn update_single(&mut self, instance: &str, record: PolledSingleRecord) -> bool {
        let mainconfig = MASTER_CONFIG.get().unwrap();

        if !self.contains(instance) {
            return false;
        }

        if record.well() {
            self.revive(&instance)
        } else if record.dead() {
            self.kill(&instance)
        } else {
            self.rest(&instance)
        };

        // i will do standard deviation stuff so that instances that are too slow will be
        // put in recovering, but not now i cba

        if self.hot.len() < mainconfig.hot_per_region as usize {
            if let Some(instance) = self.recovered.pop() {
                self.hot.push(instance);
            } else if let Some(instance) = self.stashed.pop() {
                self.hot.push(HotRecord {
                    instance,
                    weight: 1.,
                })
            }
        }

        true
    }

    pub fn contains(&self, instance: &str) -> bool {
        self.hot
            .iter()
            .chain(self.recovered.iter().chain(self.recovering.iter()))
            .map(|entry| entry.instance.address.as_str())
            .chain(
                self.dead
                    .iter()
                    .map(|entry| entry.0.instance.address.as_str()),
            )
            .chain(
                self.stashed_dead
                    .iter()
                    .map(|entry| entry.instance.address.as_str()),
            )
            .chain(
                self.stashed_recovering
                    .iter()
                    .chain(self.stashed.iter())
                    .chain(self.pending.iter())
                    .map(|entry| entry.address.as_str()),
            )
            .any(|item| item == instance)
    }

    pub fn all_instances(&self) -> Vec<&str> {
        self.hot
            .iter()
            .chain(self.recovered.iter().chain(self.recovering.iter()))
            .map(|entry| entry.instance.address.as_str())
            .chain(
                self.dead
                    .iter()
                    .map(|entry| entry.0.instance.address.as_str()),
            )
            .chain(
                self.stashed_dead
                    .iter()
                    .map(|entry| entry.instance.address.as_str()),
            )
            .chain(
                self.stashed_recovering
                    .iter()
                    .chain(self.stashed.iter())
                    .chain(self.pending.iter())
                    .map(|entry| entry.address.as_str()),
            )
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HotRecord {
    pub instance: InstanceRecord,
    pub weight: f64,
}

impl HotRecord {
    pub fn update_weight(&mut self, multiplier: f64) {
        self.weight *= multiplier;
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeadRecord {
    pub instance: InstanceRecord,
    pub dead_since: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRecord {
    pub address: String,
    pub region: String,
    pub backer: Vec<String>,
}
