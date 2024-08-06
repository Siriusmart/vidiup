use actix_web::{get, http::header::ContentType, HttpResponse};

use crate::INTERFACE_CONFIG;

#[get("/add")]
pub async fn add() -> HttpResponse {
    let options = INTERFACE_CONFIG
        .get()
        .unwrap()
        .regions_selector
        .iter()
        .map(|entry| {
            format!(
                r#"<option value="{}">{}</option>"#,
                entry.internal, entry.display
            )
        })
        .collect::<Vec<_>>()
        .join("\n          ");
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
        <h1 id="title">Add an instance</h1>
        <p id="subhead" style="width: 25em;">Paste the URL to an Invidious instance below, and select its server region.</p>
        <input type="text" id="instance" placeholder="Any Invidious URL" />
        <select id="region">
          {options}
        </select>
        <button id="add">Add instance</button>
        <p id="preview"></p>
      </div>
    </div>
  </body>
  <script src="/script/add.js"></script>
</html>"#
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}
