use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Utc;
use invidious::{ClientAsync, ClientAsyncTrait};
use serde::{Deserialize, Serialize};
use tokio::{
    task::JoinSet,
    time::{timeout, Instant},
};

use crate::{
    SavedFile, CHANNEL_ID, CONCURRENT_POLLS, INSTANCES_RECORD, INSTANCES_STATS, MASTER_CONFIG,
    OUTBOUND_CONFIG, PLAYLIST_ID, POLLING_RECORD, POLL_QUEUE, SEARCH_TERM, VIDEO_ID,
};

use super::RegionRecords;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollingRecord(pub HashMap<String, PolledSingleRecord>);

impl SavedFile for PollingRecord {
    const PATH: &'static str = ".local/share/vidiup/pollingrecords.json";
}

impl PollingRecord {
    pub fn start_poll() {
        if POLL_QUEUE.get().unwrap().lock().unwrap().is_empty() {
            return;
        }

        let mut concurrent = CONCURRENT_POLLS.get().unwrap().lock().unwrap();
        let concurrent_copy = *concurrent;
        let max = OUTBOUND_CONFIG.get().unwrap().polling.max_concurrent;
        *concurrent = max;

        for _ in 0..max - concurrent_copy {
            tokio::spawn(async {
                loop {
                    let instance = {
                        let queue = &mut *POLL_QUEUE.get().unwrap().lock().unwrap();

                        match queue.pop() {
                            Some(i) => i,
                            None => break,
                        }
                    };

                    let record = PolledSingleRecord::poll(instance.clone()).await;

                    INSTANCES_RECORD
                        .get()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .update_single(&instance, record.clone());
                    POLLING_RECORD
                        .get()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .0
                        .insert(instance, record);
                }

                let remaining = {
                    let mut remaining = CONCURRENT_POLLS.get().unwrap().lock().unwrap();
                    *remaining -= 1;
                    *remaining
                };

                if remaining == 0 {
                    let records = INSTANCES_RECORD.get().unwrap().lock().unwrap().clone();
                    let polled = POLLING_RECORD.get().unwrap().lock().unwrap().clone();
                    *INSTANCES_STATS.get().unwrap().lock().unwrap() = records.stat();
                    let _ = records.save().await;
                    let _ = polled.save().await;
                }
            });
        }
    }
}

impl PollingRecord {
    pub fn update(&mut self, instance: String, record: PolledSingleRecord) {
        self.0.insert(instance, record);
    }

    pub fn to_poll(&self, global: RegionRecords) -> Vec<String> {
        let now = Utc::now().timestamp();
        let outbound = OUTBOUND_CONFIG.get().unwrap();
        let interval = outbound.polling.interval as i64;
        let probabilities = &outbound.poll_probabilities;
        let in_queue: HashSet<String> =
            HashSet::from_iter(POLL_QUEUE.get().unwrap().lock().unwrap().clone());

        let mut hot = global
            .hot
            .into_iter()
            .filter_map(|record| {
                let address = record.instance.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        hot.sort_by_key(|item| item.1);
        hot.truncate((hot.len() as f32 * probabilities.hot).ceil() as usize);

        let mut recovered = global
            .recovered
            .into_iter()
            .filter_map(|record| {
                let address = record.instance.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        recovered.sort_by_key(|item| item.1);
        recovered.truncate((recovered.len() as f32 * probabilities.recovered).ceil() as usize);

        let mut recovering = global
            .recovering
            .into_iter()
            .filter_map(|record| {
                let address = record.instance.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        recovering.sort_by_key(|item| item.1);
        recovering.truncate((recovering.len() as f32 * probabilities.recovering).ceil() as usize);

        let mut dead = global
            .dead
            .into_iter()
            .filter_map(|record| {
                let address = record.0.instance.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        dead.sort_by_key(|item| item.1);
        dead.truncate((dead.len() as f32 * probabilities.dead).ceil() as usize);

        let mut stashed_recovering = global
            .stashed_recovering
            .into_iter()
            .filter_map(|record| {
                let address = record.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        stashed_recovering.sort_by_key(|item| item.1);
        stashed_recovering.truncate(
            (stashed_recovering.len() as f32 * probabilities.stashed_recovering).ceil() as usize,
        );

        let mut stashed_dead = global
            .stashed_dead
            .into_iter()
            .filter_map(|record| {
                let address = record.instance.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        stashed_dead.sort_by_key(|item| item.1);
        stashed_dead
            .truncate((stashed_dead.len() as f32 * probabilities.stashed_dead).ceil() as usize);

        let mut stashed = global
            .stashed
            .into_iter()
            .filter_map(|record| {
                let address = record.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        stashed.sort_by_key(|item| item.1);
        stashed.truncate((stashed.len() as f32 * probabilities.stashed).ceil() as usize);

        let mut pending = global
            .pending
            .into_iter()
            .filter_map(|record| {
                let address = record.address;
                if in_queue.contains(&address) {
                    return None;
                }
                let last_polled = self
                    .0
                    .get(&address)
                    .map(|item| item.last_polled)
                    .unwrap_or(0) as i64;
                let overdue = last_polled - now + interval;
                (overdue < 0).then_some((address, overdue))
            })
            .collect::<Vec<_>>();
        pending.sort_by_key(|item| item.1);
        pending.truncate((pending.len() as f32 * probabilities.pending).ceil() as usize);

        hot.into_iter()
            .chain(recovering)
            .chain(recovered)
            .chain(dead)
            .chain(stashed_recovering)
            .chain(stashed_dead)
            .chain(stashed)
            .chain(pending)
            .map(|item| item.0)
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PolledSingleRecord {
    #[serde(default)]
    pub last_polled: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<u32>,
}

impl PolledSingleRecord {
    pub fn score(&self) -> u32 {
        self.video.unwrap_or(0)
            + self.playlist.unwrap_or(0)
            + self.search.unwrap_or(0)
            + self.channel.unwrap_or(0)
    }

    pub fn well(&self) -> bool {
        let pollings = &OUTBOUND_CONFIG.get().unwrap().polling.features;

        !((self.video.is_none() && pollings.video)
            || (self.playlist.is_none() && pollings.playlist)
            || (self.channel.is_none() && pollings.channel)
            || (self.search.is_none() && pollings.search))
    }

    pub fn dead(&self) -> bool {
        let pollings = &OUTBOUND_CONFIG.get().unwrap().polling.features;

        (self.video.is_none() && pollings.video)
            && (self.playlist.is_none() && pollings.playlist)
            && (self.channel.is_none() && pollings.channel)
            && (self.search.is_none() && pollings.search)
    }

    pub async fn poll(instance: String) -> Self {
        let client = ClientAsync::default().instance(format!("https://{instance}"));
        let mainconfig = MASTER_CONFIG.get().unwrap();
        let outboundconfig = OUTBOUND_CONFIG.get().unwrap();

        let mut set = JoinSet::new();

        let video = Arc::new(Mutex::new(None));
        let playlist = Arc::new(Mutex::new(None));
        let channel = Arc::new(Mutex::new(None));
        let search = Arc::new(Mutex::new(None));

        if outboundconfig.polling.features.video {
            let video = video.clone();
            let client = client.clone();
            set.spawn(timeout(
                Duration::from_millis(mainconfig.timeout as u64),
                async move {
                    let start = Instant::now();
                    let id = VIDEO_ID.get().unwrap().lock().unwrap().clone();
                    if client.video(&id, None).await.is_ok() {
                        *video.lock().unwrap() = Some(start.elapsed().as_millis() as u32)
                    }
                },
            ));
        }

        if outboundconfig.polling.features.playlist {
            let playlist = playlist.clone();
            let client = client.clone();
            set.spawn(timeout(
                Duration::from_millis(mainconfig.timeout as u64),
                async move {
                    let start = Instant::now();
                    let id = PLAYLIST_ID.get().unwrap().lock().unwrap().clone();
                    if client.playlist(&id, None).await.is_ok() {
                        *playlist.lock().unwrap() = Some(start.elapsed().as_millis() as u32)
                    }
                },
            ));
        }

        if outboundconfig.polling.features.channel {
            let channel = channel.clone();
            let client = client.clone();
            set.spawn(timeout(
                Duration::from_millis(mainconfig.timeout as u64),
                async move {
                    let start = Instant::now();
                    let id = CHANNEL_ID.get().unwrap().lock().unwrap().clone();
                    if client.channel(&id, None).await.is_ok() {
                        *channel.lock().unwrap() = Some(start.elapsed().as_millis() as u32)
                    }
                },
            ));
        }

        if outboundconfig.polling.features.search {
            let search = search.clone();
            let client = client.clone();
            set.spawn(timeout(
                Duration::from_millis(mainconfig.timeout as u64),
                async move {
                    let start = Instant::now();
                    let id = SEARCH_TERM.get().unwrap().lock().unwrap().clone();
                    if client
                        .search(Some(format!("q={id}").as_str()))
                        .await
                        .is_ok()
                    {
                        *search.lock().unwrap() = Some(start.elapsed().as_millis() as u32)
                    }
                },
            ));
        }

        while set.join_next().await.is_some() {}

        let video = *video.lock().unwrap();
        let playlist = *playlist.lock().unwrap();
        let channel = *channel.lock().unwrap();
        let search = *search.lock().unwrap();

        Self {
            last_polled: Utc::now().timestamp() as u64,
            video,
            playlist,
            channel,
            search,
        }
    }
}
