use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Data {
    pub url: Option<String>,

    pub files: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ImageUrl {
    pub url: String,
    pub preview_url: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ImageUrls {
    pub images: Vec<ImageUrl>,
}
