#![feature(async_closure)]

extern crate base64;
extern crate bytes;
// use std::collections::HashMap;


use actix_web::{
 guard, http, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};

use actix_multipart::Multipart;
use actix_web::client::Client;
use std::io::Write;

use futures::{StreamExt, TryStreamExt};

mod utils;
mod models;

// use crate::utils::save_image;
// use crate::utils::generate_file_names;

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
        let preview_path_clone = image_path.clone();
        web::block(move || {
            utils::save_image(&body, image_path_clone, preview_path_clone)
        })
        .await?;
        return Ok(HttpResponse::Ok().json(models::ImageUrl{
            url: image_path,
            preview_url: preview_path
        }))
    }

    Ok(HttpResponse::Ok().into())
}
async fn images(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse, Error> {
    dbg!(req);

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        println!("{}", filename);
        let filepath = format!("./upload/{}", filename);
        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }

    Ok(HttpResponse::Ok().into())
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

// fn main() {
//     let  img = image::open("./upload/new.jpeg").unwrap();
//     let thumbnail = img.resize(120, 120, FilterType::Lanczos3);
//     let _ = thumbnail.save("out.png" ).ok().expect("Saving image failed");
// }
