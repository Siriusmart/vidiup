use std::{future::Future, pin::Pin, time::Duration};

use actix_web::{
    dev::{Service, ServiceResponse},
    http::{header::HeaderName, StatusCode},
    middleware::Logger,
    App, HttpResponseBuilder, HttpServer,
};

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
            {
                let mut instances = INSTANCES_RECORD.get().unwrap().lock().unwrap().clone();
                let record = POLLING_RECORD.get().unwrap().lock().unwrap();
                let mut to_poll = record.to_poll(instances.as_global());

                POLL_QUEUE
                    .get()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .append(&mut to_poll);
            }

            PollingRecord::start_poll();

            tokio::time::sleep(Duration::from_secs(
                OUTBOUND_CONFIG.get().unwrap().check_interval,
            ))
            .await;

            if *CONCURRENT_POLLS.get().unwrap().lock().unwrap() != 0 {
                tokio::spawn(async {
                    let instances = {
                        let instances = INSTANCES_RECORD.get().unwrap().lock().unwrap();
                        instances.clone()
                    };
                    *INSTANCES_STATS.get().unwrap().lock().unwrap() = instances.stat();
                    let _ = instances.save().await;
                });
            }
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
            .service(add::add)
    })
    .bind(("0.0.0.0", port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
