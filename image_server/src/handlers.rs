use actix_web::{web, Error, HttpResponse};

use actix_multipart::Multipart;
use actix_web::client::Client;

use futures::TryStreamExt;

use crate::{models, utils};

pub async fn images_json(data: web::Json<models::Data>) -> Result<HttpResponse, Error> {
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
        utils::save_image(body[..].into(), image_path_clone, preview_path_clone).await;
        return Ok(HttpResponse::Ok().json(models::ImageUrl {
            url: image_path[1..].to_string(),
            preview_url: preview_path[1..].to_string(),
        }));
    }
    if let Some(files) = &data.files {
        let mut images: Vec<models::ImageUrl> = Vec::new();
        for file in files.iter() {
            if let Ok(body) = base64::decode(file) {
                let (image_path, preview_path) = utils::generate_file_names();
                let image_path_clone = image_path.clone();
                let preview_path_clone = preview_path.clone();
                utils::save_image(body[..].into(), image_path_clone, preview_path_clone).await;
                images.push(models::ImageUrl {
                    preview_url: preview_path[1..].to_string(),
                    url: image_path[1..].to_string(),
                })
            }
        }
        if images.len() == 0 {
            return Ok(HttpResponse::BadRequest().into());
        }
        return Ok(HttpResponse::Ok().json(models::ImageUrls { images: images }));
    }

    Ok(HttpResponse::BadRequest().into())
}
pub async fn images(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut images: Vec<models::ImageUrl> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let body = utils::get_whole_field(&mut field).await;
        let (image_path, preview_path) = utils::generate_file_names();
        let image_path_clone = image_path.clone();
        let preview_path_clone = preview_path.clone();
        utils::save_image(body, image_path_clone, preview_path_clone).await;
        images.push(models::ImageUrl {
            preview_url: preview_path[1..].to_string(),
            url: image_path[1..].to_string(),
        })
    }
    if images.len() == 0 {
        return Ok(HttpResponse::BadRequest().into());
    }
    Ok(HttpResponse::Ok().json(models::ImageUrls { images: images }))
}

#[cfg(test)]

mod tests {
    use super::*;

    use actix_web::http::header::HeaderMap;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use bytes;
    use futures::prelude::*; // 0.3.1
    use futures::stream::StreamExt;
    use tokio::fs;

    use actix_utils::mpsc;
    use actix_web::error::PayloadError;
    use bytes::Bytes;

    use actix_web::dev::Service;
    use actix_web::{http, web, App};

    #[actix_rt::test]
    async fn test_images_json_url() {
        let url =
            "https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png"
                .to_string();
        let mut app = test::init_service(
            App::new().service(web::resource("/").route(web::post().to(images_json))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&models::Data {
                url: Some(url),
                files: None,
            })
            .to_request();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_images_json_base64files() {
        let contents = fs::read("./images/photo-1438007139926-e36a66651100.jpeg").await;
        let contents_of_file2 = fs::read("./images/photo-1495302075642-6f890b162813.jpeg").await;
        let mut app = test::init_service(
            App::new().service(web::resource("/").route(web::post().to(images_json))),
        )
        .await;
        if let Ok(contents) = contents {
            if let Ok(contents_of_file2) = contents_of_file2 {
                let req = test::TestRequest::post()
                    .uri("/")
                    .set_json(&models::Data {
                        url: None,
                        files: Some(vec![
                            base64::encode(contents),
                            base64::encode(contents_of_file2),
                        ]),
                    })
                    .to_request();
                let resp = app.call(req).await.unwrap();
                assert_eq!(resp.status(), http::StatusCode::OK);
            }
        }
    }

    fn create_stream() -> (
        mpsc::Sender<Result<Bytes, PayloadError>>,
        impl Stream<Item = Result<Bytes, PayloadError>>,
    ) {
        let (tx, rx) = mpsc::channel();

        (tx, rx.map(|res| res.map_err(|_| panic!())))
    }

    #[actix_rt::test]
    async fn test_images() {
        let headers = HeaderMap::new();
        let contents = fs::read("./images/photo-1438007139926-e36a66651100.jpeg").await;
        let (sender, payload) = create_stream();

        if let Ok(task) = contents {
            sender.send(Ok(Bytes::from(task))).unwrap();
            let multipart = Multipart::new(&headers, payload);
            let resp = images(multipart).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
