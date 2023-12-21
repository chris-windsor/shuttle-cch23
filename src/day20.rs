use actix_web::{
    post,
    web::{self, Buf},
    HttpResponse, Responder,
};
use tar::Archive;

#[post("/20/archive_files")]
pub async fn day_20_archive_files(tar_file: web::Bytes) -> impl Responder {
    let mut tar_file = Archive::new(tar_file.reader());
    let file_count = tar_file.entries().unwrap().count();

    HttpResponse::Ok().body(file_count.to_string())
}

#[post("/20/archive_files_size")]
pub async fn day_20_archive_files_size(tar_file: web::Bytes) -> impl Responder {
    let mut tar_file = Archive::new(tar_file.reader());
    let tar_files_size: u64 = tar_file
        .entries()
        .unwrap()
        .into_iter()
        .map(|file| file.unwrap().header().size().unwrap())
        .sum();

    HttpResponse::Ok().body(tar_files_size.to_string())
}
