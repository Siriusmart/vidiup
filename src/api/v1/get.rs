use actix_web::{
    get,
    web::{Json, Query},
};
use serde::{Deserialize, Serialize};

use crate::{
    PolledSingleRecord, CHANNEL_ID, INSTANCES_RECORD, OUTBOUND_CONFIG, PLAYLIST_ID, POLLING_RECORD,
    SEARCH_TERM, VIDEO_ID,
};

#[derive(Deserialize)]
struct GetQuery {
    pub region: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GetResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        address: String,
        offset: i32,
        polled: PolledSingleRecord,
        polled_on: PolledOn,
    },
    Error {
        error: String,
    },
}

#[derive(Serialize)]
pub struct PolledOn {
    pub video: String,
    pub playlist: String,
    pub channel: String,
    pub search: String,
}

impl PolledOn {
    pub fn get() -> Self {
        Self {
            video: VIDEO_ID.get().unwrap().lock().unwrap().to_string(),
            playlist: PLAYLIST_ID.get().unwrap().lock().unwrap().to_string(),
            channel: CHANNEL_ID.get().unwrap().lock().unwrap().to_string(),
            search: SEARCH_TERM.get().unwrap().lock().unwrap().to_string(),
        }
    }
}

#[get("/get")]
pub async fn get(query: Query<GetQuery>) -> Json<GetResponse> {
    let instance = if let Some(region) = &query.region {
        let offset = OUTBOUND_CONFIG.get().unwrap().offsets.0.get(region);

        if offset.is_some() {
            if let Some(region) = INSTANCES_RECORD
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .0
                .get(region)
            {
                let hot = &region.hot;

                if hot.is_empty() {
                    return Json(GetResponse::Error {
                        error: "no instance".to_string(),
                    });
                }

                let sum = hot.iter().map(|entry| entry.weight).sum::<f64>();

                let mut take = fastrand::f64() * sum;
                let mut instance = hot.len() - 1;

                for (index, record) in hot.iter().enumerate() {
                    take -= record.weight;

                    if take < 0. {
                        instance = index;
                        break;
                    }
                }

                hot[instance].instance.clone()
            } else {
                return Json(GetResponse::Error {
                    error: "no instance".to_string(),
                });
            }
        } else {
            return Json(GetResponse::Error {
                error: "no such region".to_string(),
            });
        }
    } else {
        let records = INSTANCES_RECORD.get().unwrap().lock().unwrap();
        let hot = records
            .0
            .values()
            .flat_map(|item| &item.hot)
            .collect::<Vec<_>>();

        if hot.is_empty() {
            return Json(GetResponse::Error {
                error: "no instance".to_string(),
            });
        }

        let sum = hot.iter().map(|entry| entry.weight).sum::<f64>();

        let mut take = fastrand::f64() * sum;
        let mut instance = hot.len() - 1;

        for (index, record) in hot.iter().enumerate() {
            take -= record.weight;

            if take < 0. {
                instance = index;
                break;
            }
        }

        hot[instance].instance.clone()
    };

    let offset = *OUTBOUND_CONFIG
        .get()
        .unwrap()
        .offsets
        .0
        .get(&instance.region)
        .unwrap();
    let polled = POLLING_RECORD
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .0
        .get(&instance.address)
        .unwrap()
        .clone();

    Json(GetResponse::Success {
        address: instance.address,
        offset,
        polled,
        polled_on: PolledOn::get(),
    })
}
