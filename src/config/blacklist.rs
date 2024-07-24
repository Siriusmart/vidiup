use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{SavedFile, BLACKLISTED_INSTANCES, BLACKLISTED_IP};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlackLists {
    pub ip: BlackList,
    pub instance: BlackList,
}

impl SavedFile for BlackLists {
    const PATH: &'static str = ".config/vidiup/blacklists.json";
}

impl BlackLists {
    pub fn init(&self) {
        BLACKLISTED_IP
            .set(Arc::new(Mutex::new(self.ip.hashset())))
            .unwrap();
        BLACKLISTED_INSTANCES
            .set(Arc::new(Mutex::new(self.instance.hashset())))
            .unwrap();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlackList(Vec<String>);

impl BlackList {
    pub fn hashset(&self) -> HashSet<String> {
        let mut out = HashSet::new();

        for entry in self.0.iter() {
            out.insert(entry.clone());
        }

        out
    }
}
