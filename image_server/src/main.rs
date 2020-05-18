extern crate base64;
extern crate bytes;

use actix_web::{guard, http, web, App, Error, HttpRequest, HttpResponse, HttpServer};

use actix_multipart::Multipart;
use actix_web::client::Client;

use futures::TryStreamExt;
use tokio::task::spawn_local;

mod models;
mod utils;

async fn images_json(data: web::Json<models::Data>) -> Result<HttpResponse, Error> {
    if let Some(url) = &data.url {
        let client = Client::default();
        let mut response = client
            .get(url)
            .header("User-Agent", "Actix-web")
            .send()
            .await?;
        let body = response.body().await?;
        let (image_path, preview_path) = utils::generate_file_names();
        let image_path_clone = image_path.clone();
        let preview_path_clone = preview_path.clone();
        spawn_local(async move {
            let _ = web::block(move || utils::save_image(&body, image_path_clone, preview_path_clone)).await;    
        });
        return Ok(HttpResponse::Ok().json(models::ImageUrl {
            url: image_path[1..].to_string(),
            preview_url: preview_path[1..].to_string(),
        }));
    }

    Ok(HttpResponse::Ok().into())
}
async fn images(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut images: Vec<models::ImageUrl> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let body = utils::get_whole_field(&mut field).await;
        let (image_path, preview_path) = utils::generate_file_names();
        let image_path_clone = image_path.clone();
        let preview_path_clone = preview_path.clone();
        spawn_local(async move {
            let _ = web::block(move || utils::save_image(&body, image_path_clone, preview_path_clone)).await;    
        });
        images.push(models::ImageUrl {
            preview_url: preview_path[1..].to_string(),
            url: image_path[1..].to_string(),
        })
    }
    return Ok(HttpResponse::Ok().json(models::ImageUrls { images: images }));
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/images")
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
                        .to(images),
                )
                .route(
                    web::route()
                        .guard(guard::Header("content-type", "application/json"))
                        .to(images_json),
                ),
        )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
