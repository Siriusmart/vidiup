use actix_files::NamedFile;
use actix_web::{get, web::Path, Responder};

#[get("/css/{path:.*}")]
pub async fn css(params: Path<String>) -> impl Responder {
    NamedFile::open_async(format!("./static/css/{params}")).await
}

#[get("/script/{path:.*}")]
pub async fn scripts(params: Path<String>) -> impl Responder {
    NamedFile::open_async(format!("./static/script/{params}")).await
}

#[get("/add")]
pub async fn add() -> impl Responder {
    NamedFile::open_async("./static/html/add.html").await
}
