extern crate base64;
extern crate bytes;

use actix_web::{guard, http, web, App, FromRequest, HttpServer};

mod handlers;
mod models;
pub mod utils;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/images")
                .app_data(web::Json::<models::Data>::configure(|cfg| {
                    cfg.limit(16 * 1024 * 1024)
                }))
                .route(
                    web::route()
                        .guard(guard::fn_guard(|head| {
                            head.method == http::Method::POST && {
                                if let Some(content_type) = head.headers.get("content-type") {
                                    return content_type
                                        .as_bytes()
                                        .starts_with(b"multipart/form-data");
                                }
                                false
                            }
                        }))
                        .to(handlers::images),
                )
                .route(
                    web::route()
                        .guard(guard::Header("content-type", "application/json"))
                        .to(handlers::images_json),
                ),
        )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
