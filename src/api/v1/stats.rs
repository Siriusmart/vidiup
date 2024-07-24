use actix_web::{get, web::Json};
use serde::Serialize;

use crate::INSTANCES_STATS;

#[derive(Serialize)]
struct StatsResponse {
    up: u32,
    recovering: u32,
    dead: u32,
    pending: u32
}

#[get("/stats")]
pub async fn stats() -> Json<StatsResponse> {
    let (up, recovering, dead, pending) = *INSTANCES_STATS.get().unwrap().lock().unwrap();

    Json(StatsResponse { up, recovering, dead, pending })
}
