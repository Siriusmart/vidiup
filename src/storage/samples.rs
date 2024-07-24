use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Samples {
    pub video: SampleSet,
    pub playlist: SampleSet,
    pub channel: SampleSet,
    pub search: SampleSet,
}

impl SavedFile for Samples {
    const PATH: &'static str = ".local/share/vidiup/sampleset.json";
}

impl Samples {
    pub fn init(&self) {
        VIDEO_ID
            .set(Arc::new(Mutex::new(self.video.gen())))
            .unwrap();
        PLAYLIST_ID
            .set(Arc::new(Mutex::new(self.playlist.gen())))
            .unwrap();
        CHANNEL_ID
            .set(Arc::new(Mutex::new(self.channel.gen())))
            .unwrap();
        SEARCH_TERM
            .set(Arc::new(Mutex::new(self.search.gen())))
            .unwrap();
    }

    pub fn gen(&self) {
        let video = self.video.gen();
        let playlist = self.playlist.gen();
        let channel = self.channel.gen();
        let search = self.search.gen();

        *VIDEO_ID.get().unwrap().lock().unwrap() = video;
        *PLAYLIST_ID.get().unwrap().lock().unwrap() = playlist;
        *CHANNEL_ID.get().unwrap().lock().unwrap() = channel;
        *SEARCH_TERM.get().unwrap().lock().unwrap() = search;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SampleSet(Vec<String>);

impl SampleSet {
    pub fn gen(&self) -> String {
        if self.0.is_empty() {
            panic!("sample set cannot be empty");
        }

        self.0[fastrand::usize(..self.0.len())].to_string()
    }
}
