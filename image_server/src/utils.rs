extern crate uuid;
extern crate image;
use uuid::Uuid;
use image::imageops::FilterType;


const UPLOAD_DIR: &str = ".//uploads/";

pub fn generate_file_names() -> (String, String) {    
    let uuid = Uuid::new_v4();
    let name = uuid.to_hyphenated_ref();
    return (
        format!("{}{}.jpeg", UPLOAD_DIR, name),
        format!("{}{}_preview.jpeg", UPLOAD_DIR, name),
    );
}

pub fn save_image(body: &[u8], image_path: String, preview_path: String) -> std::result::Result<(), image::ImageError> {
    let img = image::load_from_memory(body).unwrap();
    img.save(image_path)?;
    let thumbnail = img.resize(120, 120, FilterType::Lanczos3);
    thumbnail.save(preview_path)
}