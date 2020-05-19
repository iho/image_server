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
        if images.is_empty() {
            return Ok(HttpResponse::BadRequest().into());
        }
        return Ok(HttpResponse::Ok().json(models::ImageUrls { images }));
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
    if images.is_empty() {
        return Ok(HttpResponse::BadRequest().into());
    }
    Ok(HttpResponse::Ok().json(models::ImageUrls { images }))
}

#[cfg(test)]

mod tests {
    use super::*;

    use actix_web::http::StatusCode;
    use bytes;
    use futures::prelude::*; // 0.3.1
    use futures::stream::StreamExt;
    use tokio::fs;

    use actix_utils::mpsc;
    use actix_web::error::PayloadError;
    use actix_web::{http, web};
    use bytes::Bytes;
    use std::time::Duration;
    use tokio::time;

    #[actix_rt::test]
    async fn test_images_json_url() {
        let url =
            "https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png"
                .to_string();

        let req = web::Json(models::Data {
            url: Some(url),
            files: None,
        });
        let resp = images_json(req).await.unwrap();
        time::delay_for(Duration::from_millis(5000)).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_images_json_base64files() {
        let contents = fs::read("./images/photo-1438007139926-e36a66651100.jpeg").await;
        let contents_of_file2 = fs::read("./images/photo-1495302075642-6f890b162813.jpeg").await;

        if let Ok(contents) = contents {
            if let Ok(contents_of_file2) = contents_of_file2 {
                let req = web::Json(models::Data {
                    url: None,
                    files: Some(vec![
                        base64::encode(contents),
                        base64::encode(contents_of_file2),
                    ]),
                });
                let resp = images_json(req).await.unwrap();

                time::delay_for(Duration::from_millis(5000)).await;
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
        use actix_web::http::header::{self, HeaderMap};
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(
                "multipart/mixed; boundary=\"6cbe132b97ae69dc2aaf6eeffd7ca9f0\"",
            ),
        );
        let contents = fs::read("./images/test.body").await;
        let (sender, payload) = create_stream();

        if let Ok(task) = contents {
            sender.send(Ok(Bytes::from(task))).unwrap();
            let multipart = Multipart::new(&headers, payload);
            let resp = images(multipart).await.unwrap();
            time::delay_for(Duration::from_millis(5000)).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
