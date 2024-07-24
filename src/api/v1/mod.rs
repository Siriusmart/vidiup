use actix_web::Scope;
mod add;
mod get;
mod regions;
mod stats;

pub fn scope() -> Scope {
    Scope::new("/v1")
        .service(get::get)
        .service(regions::regions)
        .service(add::add)
        .service(stats::stats)
}
