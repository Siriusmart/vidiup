use actix_web::{
    get,
    web::{Json, Query},
    HttpRequest,
};
use serde::{Deserialize, Serialize};

use crate::{INSTANCES_RECORD, OUTBOUND_CONFIG};

#[derive(Deserialize)]
struct AddQuery {
    pub region: String,
    pub instance: String,
}

#[derive(Serialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
enum AddResponse {
    Success { address: String, region: String },
    Error { error: String },
}

#[get("/add")]
async fn add(query: Query<AddQuery>, req: HttpRequest) -> Json<AddResponse> {
    if !OUTBOUND_CONFIG
        .get()
        .unwrap()
        .offsets
        .0
        .contains_key(&query.region)
    {
        return Json(AddResponse::Error {
            error: "no such region".to_string(),
        });
    }

    INSTANCES_RECORD.get().unwrap().lock().unwrap().add(
        query.instance.to_string(),
        query.region.to_string(),
        req.connection_info()
            .realip_remote_addr()
            .unwrap()
            .to_string(),
    );

    Json(AddResponse::Success {
        address: query.instance.to_string(),
        region: query.region.to_string(),
    })
}
