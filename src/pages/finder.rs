use actix_web::{
    body::BoxBody,
    get,
    http::header::ContentType,
    web::{Query, Redirect},
    HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;

use crate::{
    RegionSelectorEntry, INSTANCES_RECORD, INTERFACE_CONFIG, OUTBOUND_CONFIG, POLLING_RECORD,
};

#[derive(Deserialize)]
struct GetQuery {
    pub region: Option<String>,
}

fn selector(selected: &Option<String>) -> String {
    let mut selectors = vec![if selected.is_some() {
        r#"<a href="/finder">All regions</a>"#.to_string()
    } else {
        r#"<span>All regions</span>"#.to_string()
    }];

    for RegionSelectorEntry { display, internal } in
        INTERFACE_CONFIG.get().unwrap().regions_selector.iter()
    {
        if selected.as_ref() == Some(internal) {
            selectors.push(format!("<span>{display}</span>"))
        } else {
            selectors.push(format!(r#"<a href="?region={internal}">{display}</a>"#))
        }
    }

    format!(
        r#"<div id="regions">
            {}
        </div>
        "#,
        selectors.join("\n            ")
    )
}

fn construct(body: &str, selector: &str) -> String {
    format!(
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
        <h1 id="title">Find me an instance</h1>
        {body}
        {selector}
      </div>
    </div>
  </body>
</html>"#
    )
}

const NO_INSTANCE: &str = r#"<p id="subhead">
        No instances found.
        </p>
        <table id="record">
          <tr>
            <th>Video</th>
            <th>Playlist</th>
            <th>Channel</th>
            <th>Search</th>
          </tr>
          <tr>
            <td class="inactive">-ms</td>
            <td class="inactive">-ms</td>
            <td class="inactive">-ms</td>
            <td class="inactive">-ms</td>
          </tr>
        </table>"#;

#[get("/finder")]
pub async fn finder(query: Query<GetQuery>, req: HttpRequest) -> HttpResponse {
    if query.region.as_ref().is_some_and(|region| {
        !OUTBOUND_CONFIG
            .get()
            .unwrap()
            .offsets
            .0
            .contains_key(region)
    }) {
        Redirect::to("/finder")
            .permanent()
            .respond_to(&req)
            .set_body(BoxBody::new(""))
    } else {
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(finder_task(query).await)
    }
}

async fn finder_task(query: Query<GetQuery>) -> String {
    let instance = if let Some(region) = &query.region {
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
                return construct(NO_INSTANCE, &selector(&query.region));
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
            return construct(NO_INSTANCE, &selector(&query.region));
        }
    } else {
        let records = INSTANCES_RECORD.get().unwrap().lock().unwrap();
        let hot = records
            .0
            .values()
            .into_iter()
            .flat_map(|item| &item.hot)
            .collect::<Vec<_>>();

        if hot.is_empty() {
            return construct(NO_INSTANCE, &selector(&query.region));
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
    let record = POLLING_RECORD
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .instances
        .get(&instance.address)
        .unwrap()
        .clone();

    let instance = instance.address;

    let mut headers = Vec::new();
    let mut stats = Vec::new();

    if OUTBOUND_CONFIG.get().unwrap().polling.features.video {
        headers.push(format!("<th>Video</th>"));
        if let Some(latency) = record.video {
            stats.push(format!(
                r#"<td class="{}">{}ms</td>"#,
                INTERFACE_CONFIG
                    .get()
                    .unwrap()
                    .latency_thresholds
                    .quality(latency),
                latency.saturating_add_signed(offset)
            ));
        } else {
            stats.push(r#"<td class="inactive">-ms</td>"#.to_string());
        }
    }

    if OUTBOUND_CONFIG.get().unwrap().polling.features.playlist {
        headers.push(format!("<th>Playlist</th>"));
        if let Some(latency) = record.playlist {
            stats.push(format!(
                r#"<td class="{}">{}ms</td>"#,
                INTERFACE_CONFIG
                    .get()
                    .unwrap()
                    .latency_thresholds
                    .quality(latency),
                latency.saturating_add_signed(offset)
            ));
        } else {
            stats.push(r#"<td class="inactive">-ms</td>"#.to_string());
        }
    }

    if OUTBOUND_CONFIG.get().unwrap().polling.features.channel {
        headers.push(format!("<th>Channel</th>"));
        if let Some(latency) = record.channel {
            stats.push(format!(
                r#"<td class="{}">{}ms</td>"#,
                INTERFACE_CONFIG
                    .get()
                    .unwrap()
                    .latency_thresholds
                    .quality(latency),
                latency.saturating_add_signed(offset)
            ));
        } else {
            stats.push(r#"<td class="inactive">-ms</td>"#.to_string());
        }
    }

    if OUTBOUND_CONFIG.get().unwrap().polling.features.search {
        headers.push(format!("<th>Search</th>"));
        if let Some(latency) = record.search {
            stats.push(format!(
                r#"<td class="{}">{}ms</td>"#,
                INTERFACE_CONFIG
                    .get()
                    .unwrap()
                    .latency_thresholds
                    .quality(latency),
                latency.saturating_add_signed(offset)
            ));
        } else {
            stats.push(r#"<td class="inactive">-ms</td>"#.to_string());
        }
    }

    let headers = headers.join("\n            ");
    let stats = stats.join("\n            ");

    let html = format!(
        r#"<p id="subhead">
        Instance: <a href="https://{instance}" target="_blank" id="foundinstance"><i>{instance}</i></a>
        </p>
        <table id="record">
          <tr>
            {headers}
          </tr>
          <tr>
            {stats}
          </tr>
        </table>"#
    );
    construct(&html, &selector(&query.region))
}
