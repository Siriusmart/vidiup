use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use invidious::{ClientAsync, ClientAsyncTrait};
use serde::{Deserialize, Serialize};
use tokio::{
    task::JoinSet,
    time::{timeout, Instant},
};

use crate::{
    SavedFile, CHANNEL_ID, MASTER_CONFIG, OUTBOUND_CONFIG, PLAYLIST_ID, SEARCH_TERM, VIDEO_ID,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollingRecord {
    pub last_polled: u64,
    pub instances: HashMap<String, PolledSingleRecord>,
}

impl SavedFile for PollingRecord {
    const PATH: &'static str = ".local/share/vidiup/pollingrecords.json";
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolledSingleRecord {
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

        while let Some(_) = set.join_next().await {}

        let video = *video.lock().unwrap();
        let playlist = *playlist.lock().unwrap();
        let channel = *channel.lock().unwrap();
        let search = *search.lock().unwrap();

        Self {
            video,
            playlist,
            channel,
            search,
        }
    }
}
