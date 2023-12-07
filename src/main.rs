use std::{collections::HashMap, path::PathBuf};

use actix_web::{
    get,
    http::header::HeaderValue,
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder, Result,
};
use base64::{engine::general_purpose, Engine};
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

#[post("/6")]
async fn day_6(body: web::Bytes) -> Result<impl Responder> {
    let mut elf_on_shelf_count = 0;

    let doc: Vec<_> = body
        .windows(3)
        .enumerate()
        .filter_map(|(pos, chunk)| {
            if String::from_utf8(chunk.to_vec()).unwrap_or_default() == "elf" {
                if String::from_utf8(body.slice(pos..).to_vec())
                    .unwrap_or_default()
                    .starts_with("elf on a shelf")
                {
                    elf_on_shelf_count += 1;
                }

                return Some("elf");
            }
            None
        })
        .collect();

    let shelves: Vec<_> = body
        .windows(5)
        .enumerate()
        .filter_map(|(pos, chunk)| {
            if String::from_utf8(chunk.to_vec()).unwrap_or_default() == "shelf" {
                if (String::from_utf8(body.slice(..pos).to_vec()))
                    .unwrap_or_default()
                    .ends_with("elf on a ")
                {
                    return Some("elf on a shelf");
                }
            }
            None
        })
        .collect();

    let elf_count = doc.len();
    let shelf_no_elf_count = shelves.len();

    Ok(web::Json(json!({
        "elf": elf_count,
        "elf on a shelf": elf_on_shelf_count,
        "shelf with no elf on it": shelf_no_elf_count
    })))
}

fn decode_cookie_header(cookie_header: &HeaderValue) -> String {
    let header_value = cookie_header.to_str().unwrap();
    let recipe_encoded = &header_value["recipe=".len()..];

    String::from_utf8(general_purpose::STANDARD.decode(recipe_encoded).unwrap()).unwrap()
}

#[get("/7/decode")]
async fn day_7_decode(req: HttpRequest) -> impl Responder {
    let cookie_header = req.headers().get("Cookie").unwrap();
    let recipe = decode_cookie_header(&cookie_header);

    HttpResponse::Ok().body(recipe)
}

#[derive(Debug, Deserialize)]
struct BakeOrder {
    recipe: serde_json::Value,
    pantry: serde_json::Value,
}

#[get("/7/bake")]
async fn day_7_bake(req: HttpRequest) -> Result<impl Responder> {
    let cookie_header = req.headers().get("Cookie").unwrap();
    let recipe = decode_cookie_header(&cookie_header);

    let bake_order: BakeOrder = serde_json::from_str(recipe.as_str()).unwrap();
    let recipe = bake_order.recipe.as_object().unwrap();
    let pantry = bake_order.pantry.as_object().unwrap();

    let mut calc: Vec<_> = recipe
        .iter()
        .map(|(item, qty)| {
            (
                item,
                &pantry
                    .get(item)
                    .unwrap_or(&serde_json::Value::from(0))
                    .as_u64()
                    .unwrap()
                    / qty.as_u64().unwrap(),
            )
        })
        .collect();

    calc.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let max_cookies = calc.get(0).unwrap().1;

    let pantry_balances: HashMap<_, _> = pantry
        .into_iter()
        .map(|(item, qty)| {
            (
                item,
                qty.as_u64().unwrap_or_default()
                    - (max_cookies
                        * &recipe
                            .get(item)
                            .unwrap_or(&serde_json::Value::from(0))
                            .as_u64()
                            .unwrap()),
            )
        })
        .collect();

    Ok(web::Json(json!({
        "cookies": max_cookies,
        "pantry": pantry_balances
    })))
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(fake_error);
        cfg.service(day_1);
        cfg.service(day_4_strength);
        cfg.service(day_4_contest);
        cfg.service(day_6);
        cfg.service(day_7_decode);
        cfg.service(day_7_bake);
    };

    Ok(config.into())
}
