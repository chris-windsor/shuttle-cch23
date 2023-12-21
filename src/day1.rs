use std::path::PathBuf;

use actix_web::{get, web, HttpResponse, Responder};

#[get("/1/{tail:.*}")]
pub async fn day_1(path: web::Path<PathBuf>) -> impl Responder {
    let packet_ids = path.into_inner();
    let packet_ids = packet_ids.as_path();

    let xor_res = packet_ids
        .iter()
        .map(|packet_id| {
            packet_id
                .to_str()
                .unwrap()
                .parse::<i32>()
                .expect("to parse path")
        })
        .reduce(|acc, cur| acc ^ cur)
        .unwrap();

    let pow_res = xor_res.pow(3);

    HttpResponse::Ok().body(pow_res.to_string())
}
