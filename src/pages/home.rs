use actix_web::{get, http::header::ContentType, HttpResponse};

use crate::INSTANCES_STATS;

#[get("/")]
pub async fn home() -> HttpResponse {
    let (up, recovering, dead, pending) = *INSTANCES_STATS.get().unwrap().lock().unwrap();

    let html = format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link
      href="https://fonts.googleapis.com/css2?family=Open+Sans:ital,wght@0,300..800;1,300..800&family=Roboto+Mono:ital,wght@0,100..700;1,100..700&display=swap"
      rel="stylesheet"
    />
    <title>VidiUp - Invidious Health</title>
    <link rel="stylesheet" href="/css/home.css" />
  </head>
  <body>
    <div class="fullpage">
      <div id="mainscreen">
        <h1 id="title">Is Invidious Down?</h1>
        <p id="stats">
          <span id="up"><span class="counter">{up}</span> up</span>,
          <span id="recovering"><span class="counter">{recovering}</span> recovering</span
          >, <span id="dead"><span class="counter">{dead}</span> dead</span>.
          <span id="pending" style="display:inline-block;">(<span class="counter">{pending}</span> pending)</span>
        </p>
        <div id="urls">
          <a href="/finder">Find me an instance</a>
          <a href="https://docs.invidious.io/installation/" target="_blank">Host an instance</a>
          <a href="/add">Add an instance</a>
        </div>
      </div>
    </div>
  </body>
</html>"#
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}
