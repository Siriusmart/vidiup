use std::{
    collections::HashSet,
    sync::{Arc, Mutex, OnceLock},
};

use crate::*;

// configs
pub static MASTER_CONFIG: OnceLock<MasterConfig> = OnceLock::new();
pub static OUTBOUND_CONFIG: OnceLock<OutboundConfig> = OnceLock::new();
pub static INTERFACE_CONFIG: OnceLock<InterfaceConfig> = OnceLock::new();
pub static BLACKLISTS: OnceLock<Arc<Mutex<BlackLists>>> = OnceLock::new();

// storages
pub static POLLING_RECORD: OnceLock<Arc<Mutex<PollingRecord>>> = OnceLock::new();
pub static INSTANCES_RECORD: OnceLock<Arc<Mutex<InstancesRecords>>> = OnceLock::new();
pub static SAMPLESETS: OnceLock<Arc<Mutex<Samples>>> = OnceLock::new();

// generated samples
pub static VIDEO_ID: OnceLock<Arc<Mutex<String>>> = OnceLock::new();
pub static PLAYLIST_ID: OnceLock<Arc<Mutex<String>>> = OnceLock::new();
pub static CHANNEL_ID: OnceLock<Arc<Mutex<String>>> = OnceLock::new();
pub static SEARCH_TERM: OnceLock<Arc<Mutex<String>>> = OnceLock::new();

pub static BLACKLISTED_IP: OnceLock<Arc<Mutex<HashSet<String>>>> = OnceLock::new();
pub static BLACKLISTED_INSTANCES: OnceLock<Arc<Mutex<HashSet<String>>>> = OnceLock::new();

#[allow(clippy::type_complexity)]
pub static INSTANCES_STATS: OnceLock<Arc<Mutex<(u32, u32, u32, u32)>>> = OnceLock::new();
pub static CONCURRENT_POLLS: OnceLock<Arc<Mutex<u32>>> = OnceLock::new();
pub static POLL_QUEUE: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

pub async fn init() {
    let _ = MASTER_CONFIG.set(MasterConfig::load().await.unwrap());
    let _ = OUTBOUND_CONFIG.set(OutboundConfig::load().await.unwrap());
    let _ = INTERFACE_CONFIG.set(InterfaceConfig::load().await.unwrap());
    let _ = BLACKLISTS.set(Arc::new(Mutex::new(BlackLists::load().await.unwrap())));

    let _ = POLLING_RECORD.set(Arc::new(Mutex::new(PollingRecord::load().await.unwrap())));
    let _ = INSTANCES_RECORD.set(Arc::new(Mutex::new(
        InstancesRecords::load().await.unwrap(),
    )));
    let _ = SAMPLESETS.set(Arc::new(Mutex::new(Samples::load().await.unwrap())));

    BLACKLISTS.get().unwrap().lock().unwrap().init();
    SAMPLESETS.get().unwrap().lock().unwrap().init();
    INSTANCES_STATS
        .set(Arc::new(Mutex::new(
            INSTANCES_RECORD.get().unwrap().lock().unwrap().stat(),
        )))
        .unwrap();

    CONCURRENT_POLLS.set(Arc::new(Mutex::new(0))).unwrap();
    POLL_QUEUE.set(Arc::new(Mutex::new(Vec::new()))).unwrap();
}
