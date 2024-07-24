use std::{future::Future, pin::Pin, time::Duration};

use actix_web::{
    dev::{Service, ServiceResponse},
    http::{header::HeaderName, StatusCode},
    middleware::Logger,
    App, HttpResponseBuilder, HttpServer,
};
use chrono::Utc;
use log::info;
use simplelog::Config;
use vidiup::*;

#[tokio::main]
async fn main() {
    init().await;

    let port = MASTER_CONFIG.get().unwrap().port;
    let reverse_proxy = MASTER_CONFIG.get().unwrap().reverse_proxy;

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    tokio::spawn(async {
        loop {
            let next_poll = POLLING_RECORD.get().unwrap().lock().unwrap().last_polled
                + OUTBOUND_CONFIG.get().unwrap().polling.interval;
            let now = Utc::now().timestamp() as u64;

            if next_poll > now {
                tokio::time::sleep(Duration::from_secs(next_poll - now)).await;
            }

            let instances = INSTANCES_RECORD.get().unwrap().lock().unwrap().clone();
            instances.poll().await;
            INSTANCES_RECORD.get().unwrap().lock().unwrap().update();
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap_fn(
                move |mut req,
                      srv|
                      -> Pin<
                    Box<dyn Future<Output = Result<ServiceResponse, actix_web::Error>>>,
                > {
                    let address = if !reverse_proxy {
                        req.peer_addr().map(|addr| addr.ip().to_string())
                    } else {
                        req.connection_info()
                            .realip_remote_addr()
                            .map(|addr| addr.to_string())
                    };

                    let blacklist = BLACKLISTED_IP.get().unwrap().lock().unwrap();
                    let allowed = address.as_ref().is_some_and(|ip| !blacklist.contains(ip));

                    if allowed {
                        if !reverse_proxy {
                            let headers = req.headers_mut();
                            headers.remove(HeaderName::from_static("Forwarded"));
                            headers.remove(HeaderName::from_static("X-Forwarded-For"));
                        }
                        srv.call(req)
                    } else {
                        info!(
                            "Blocked access attempt from {}",
                            address.unwrap_or("no address".to_string())
                        );
                        Box::pin(async {
                            Ok(ServiceResponse::new(
                                req.into_parts().0,
                                HttpResponseBuilder::new(StatusCode::FORBIDDEN).body("Forbidden"),
                            ))
                        }) as _
                    }
                },
            )
            .wrap(if reverse_proxy {
                Logger::new(r#"%{Forwarded}i "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#)
            } else {
                Logger::default()
            })

            .service(api::scope())
            .service(css)
            .service(scripts)
            .service(home::home)
            .service(finder::finder)
            .service(add)
    })
    .bind(("0.0.0.0", port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
