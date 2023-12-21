use std::path::PathBuf;

use actix_files::NamedFile;
use actix_multipart::form::{bytes::Bytes, MultipartForm};
use actix_web::{
    get, post,
    web::{self},
    Responder,
};
use image::GenericImageView;

#[get("/11/assets/{filename:.*}")]
pub async fn day_11_image(path: web::Path<PathBuf>) -> impl Responder {
    let asset_path = path.into_inner();
    let asset_path = asset_path.as_path();

    NamedFile::open_async(format!("assets/{}", asset_path.display())).await
}

#[derive(MultipartForm)]
struct ImageForm {
    image: Bytes,
}

#[post("/11/red_pixels")]
pub async fn day_11_red_pixels(MultipartForm(form): MultipartForm<ImageForm>) -> impl Responder {
    let img = image::load_from_memory(&form.image.data).unwrap();

    let magical_red = img
        .pixels()
        .filter(|(_x, _y, rgba)| {
            let [r, g, b, _a] = rgba.0;

            r as u16 > (g as u16 + b as u16)
        })
        .count();

    magical_red.to_string()
}
