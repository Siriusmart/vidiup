use actix_web::{get, web::Json};

use crate::OUTBOUND_CONFIG;

#[get("/regions")]
pub async fn regions() -> Json<Vec<String>> {
    Json(
        OUTBOUND_CONFIG
            .get()
            .unwrap()
            .offsets
            .0
            .keys()
            .into_iter()
            .map(String::clone)
            .collect(),
    )
}
