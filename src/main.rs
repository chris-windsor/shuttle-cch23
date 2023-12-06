use std::path::PathBuf;

use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    HttpResponse, Responder, Result,
};
use serde::Deserialize;
use serde_json::json;
use shuttle_actix_web::ShuttleActixWeb;

#[get("/-1/error")]
async fn fake_error() -> impl Responder {
    HttpResponse::InternalServerError()
}

#[get("/1/{tail:.*}")]
async fn day_1(path: web::Path<PathBuf>) -> impl Responder {
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

#[derive(Clone, Default, Deserialize)]
struct Reindeer {
    name: String,
    strength: i32,
    #[serde(default)]
    speed: f32,
    #[serde(default)]
    height: i32,
    #[serde(default)]
    antler_width: i32,
    #[serde(default)]
    snow_magic_power: i32,
    #[serde(default)]
    favorite_food: String,
    #[serde(default, rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: i32,
}

#[post("/4/strength")]
async fn day_4_strength(reindeer: web::Json<Vec<Reindeer>>) -> impl Responder {
    let group_strength: i32 = reindeer.0.iter().map(|deer| deer.strength).sum();

    HttpResponse::Ok().body(group_strength.to_string())
}

#[derive(Default)]
struct ContestResults {
    fastest: Reindeer,
    tallest: Reindeer,
    magician: Reindeer,
    consumer: Reindeer,
}

#[post("/4/contest")]
async fn day_4_contest(reindeer: web::Json<Vec<Reindeer>>) -> Result<impl Responder> {
    let results = reindeer
        .0
        .iter()
        .fold(ContestResults::default(), |mut results, deer| {
            if results.fastest.speed < deer.speed {
                results.fastest = deer.clone()
            }
            if results.tallest.height < deer.height {
                results.tallest = deer.clone()
            }
            if results.magician.snow_magic_power < deer.snow_magic_power {
                results.magician = deer.clone();
            }
            if results.consumer.candies_eaten_yesterday < deer.candies_eaten_yesterday {
                results.consumer = deer.clone();
            }

            results
        });

    Ok(web::Json(json!({
      "fastest": format!("Speeding past the finish line with a strength of {} is {}", results.fastest.strength, results.fastest.name),
      "tallest": format!("{} is standing tall with his {} cm wide antlers", results.tallest.name, results.tallest.antler_width),
      "magician": format!("{} could blast you away with a snow magic power of {}", results.magician.name, results.magician.snow_magic_power),
      "consumer": format!("{} ate lots of candies, but also some {}", results.consumer.name, results.consumer.favorite_food)
    })))
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(fake_error);
        cfg.service(day_1);
        cfg.service(day_4_strength);
        cfg.service(day_4_contest);
    };

    Ok(config.into())
}
