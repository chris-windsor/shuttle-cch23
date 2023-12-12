use std::{collections::HashMap, path::PathBuf};

use actix_files::NamedFile;
use actix_multipart::form::{bytes::Bytes, MultipartForm};
use actix_web::{
    get,
    http::header::HeaderValue,
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder, Result,
};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use image::GenericImageView;
use serde::Deserialize;
use serde_json::json;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_persist::PersistInstance;
use ulid::Ulid;
use uuid::Uuid;

#[get("/")]
async fn base() -> impl Responder {
    HttpResponse::Ok()
}

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
    let doc: Vec<_> = body
        .windows(3)
        .enumerate()
        .filter_map(|(_pos, chunk)| {
            if String::from_utf8(chunk.to_vec()).unwrap_or_default() == "elf" {
                return Some("elf");
            }
            None
        })
        .collect();

    let elf_count = doc.len();

    let (elf_on_shelf_count, shelf_no_elf_count) = body
        .windows(5)
        .enumerate()
        .filter_map(|(pos, chunk)| {
            if String::from_utf8(chunk.to_vec()).unwrap_or_default() == "shelf" {
                if (String::from_utf8(body.slice(..pos).to_vec()))
                    .unwrap_or_default()
                    .ends_with("elf on a ")
                {
                    return Some(&true);
                } else {
                    return Some(&false);
                }
            }
            None
        })
        .fold((0, 0), |(yay, nay), cur| {
            // woah. cool
            (yay + (cur == &true) as i32, nay + (cur == &false) as i32)
        });

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
        // get rid of ingredients that are listed but have 0 qty
        .filter(|(_item, qty)| qty.as_i64().unwrap() > 0)
        .map(|(item, qty)| {
            (
                item,
                pantry
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

#[derive(Deserialize)]
struct Pokemon {
    weight: f32,
}

async fn get_pokemon_by_id(id: u32) -> Pokemon {
    let pokemon = reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{}", id))
        .await
        .expect("make request");
    let pokemon = pokemon.json::<Pokemon>().await.expect("parse json");

    pokemon
}

#[get("/8/weight/{pokedex_number}")]
async fn day_8_weight(path: web::Path<u32>) -> impl Responder {
    let pokedex_number = path.into_inner();

    let pokemon = get_pokemon_by_id(pokedex_number).await;

    HttpResponse::Ok().body((pokemon.weight / 10.0).to_string())
}

#[get("/8/drop/{pokedex_number}")]
async fn day_8_drop(path: web::Path<u32>) -> impl Responder {
    let pokedex_number = path.into_inner();

    let pokemon = get_pokemon_by_id(pokedex_number).await;

    let time = f32::sqrt(2.0 * 10.0 / 9.825);
    let velocity = 9.825 * time;
    let momentum: f32 = (pokemon.weight as f32) * velocity / 10.0;

    HttpResponse::Ok().body(momentum.to_string())
}

#[get("/11/assets/{filename:.*}")]
async fn day_11_image(path: web::Path<PathBuf>) -> impl Responder {
    let asset_path = path.into_inner();
    let asset_path = asset_path.as_path();

    NamedFile::open_async(format!("assets/{}", asset_path.display())).await
}

#[derive(MultipartForm)]
struct ImageForm {
    image: Bytes,
}

#[post("/11/red_pixels")]
async fn day_11_red_pixels(MultipartForm(form): MultipartForm<ImageForm>) -> impl Responder {
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

#[post("/12/save/{packet}")]
async fn day_12_save(packet: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let packet = packet.into_inner();

    let _ = data.persist.save::<NaiveTime>(&packet, Utc::now().time());

    HttpResponse::Ok()
}

#[get("/12/load/{packet}")]
async fn day_12_load(packet: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let packet = packet.into_inner();

    let current_time = Utc::now().time();
    let packet_time = data.persist.load::<NaiveTime>(&packet).unwrap();

    let diff = current_time - packet_time.clone();

    diff.num_seconds().to_string()
}

#[post("/12/ulids")]
async fn day_12_ulids(ulids: web::Json<Vec<Ulid>>) -> Result<impl Responder> {
    let uuids = ulids
        .0
        .iter()
        .map(|ulid| Uuid::from(ulid.clone()))
        .rev()
        .collect::<Vec<Uuid>>();

    Ok(web::Json(json!(uuids)))
}

#[post("/12/ulids/{weekday}")]
async fn day_12_lsb(weekday: web::Path<u8>, ulids: web::Json<Vec<Ulid>>) -> Result<impl Responder> {
    let weekday = weekday.into_inner();
    let weekday = Weekday::try_from(weekday).unwrap();

    let (christmas_eve_count, weekday_count, future_count, lsb_count) = ulids.0.iter().fold(
        (0, 0, 0, 0),
        |(christmas_eve_cur, weekday_cur, future_cur, lsb_cur), ulid| {
            let current_time = Utc::now();
            let ulid_time: DateTime<Utc> = ulid.datetime().into();

            let month_of_year = ulid_time.month();
            let day_of_month = ulid_time.day();
            let matches_christmas_eve = month_of_year == 12 && day_of_month == 24;

            let matches_weekday = ulid_time.weekday() == weekday;

            let matches_future = (ulid_time - current_time).num_seconds() > 0;

            let matches_lsb = ulid.random() % (1 << 1) == 1;

            (
                christmas_eve_cur + (matches_christmas_eve as i32),
                weekday_cur + (matches_weekday as i32),
                future_cur + (matches_future as i32),
                lsb_cur + (matches_lsb as i32),
            )
        },
    );

    Ok(web::Json(json!({
        "christmas eve": christmas_eve_count,
        "weekday": weekday_count,
        "in the future": future_count,
        "LSB is 1": lsb_count
    })))
}

struct AppState {
    persist: PersistInstance,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(base);
        cfg.service(fake_error);
        cfg.service(day_1);
        cfg.service(day_4_strength);
        cfg.service(day_4_contest);
        cfg.service(day_6);
        cfg.service(day_7_decode);
        cfg.service(day_7_bake);
        cfg.service(day_8_weight);
        cfg.service(day_8_drop);
        cfg.service(day_11_image);
        cfg.service(day_11_red_pixels);
        cfg.service(day_12_save);
        cfg.service(day_12_load);
        cfg.service(day_12_ulids);
        cfg.service(day_12_lsb);

        // 32MB
        cfg.app_data(web::PayloadConfig::new(1 << 25));

        let app_data = web::Data::new(AppState { persist });
        cfg.app_data(app_data.clone());
    };

    Ok(config.into())
}
