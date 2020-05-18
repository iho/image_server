extern crate image;
extern crate uuid;
use actix_multipart::Field;
use actix_web::web;
use bytes::BytesMut;
use futures::StreamExt;
use image::imageops::FilterType;
use tokio::task::spawn_local;
use uuid::Uuid;

const UPLOAD_DIR: &str = "./uploads/";

pub fn generate_file_names() -> (String, String) {
    let uuid = Uuid::new_v4();
    let name = uuid.to_hyphenated_ref();
    return (
        format!("{}{}.jpeg", UPLOAD_DIR, name),
        format!("{}{}_preview.jpeg", UPLOAD_DIR, name),
    );
}

pub async fn save_image(body: bytes::BytesMut, image_path: String, preview_path: String) {
    spawn_local(async move {
        let _ = web::block(move || {
            let img = image::load_from_memory(&body)?;
            img.save(image_path)?;
            let thumbnail = img.resize(120, 120, FilterType::Lanczos3);
            let result = thumbnail.save(preview_path);
            result
        })
        .await;
    });
}

pub async fn get_whole_field(field: &mut Field) -> BytesMut {
    let mut b = BytesMut::new();
    loop {
        match field.next().await {
            Some(Ok(chunk)) => b.extend_from_slice(&chunk),
            None => return b,
            _ => unreachable!(),
        }
    }
}
