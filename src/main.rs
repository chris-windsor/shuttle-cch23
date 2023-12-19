use std::{collections::HashMap, path::PathBuf};

use actix_files::NamedFile;
use actix_multipart::form::{bytes::Bytes, MultipartForm};
use actix_web::{
    error, get,
    http::{
        header::{ContentType, HeaderValue},
        StatusCode,
    },
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder, Result,
};
use askama::Template;
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use derive_more::{Display, Error};
use image::GenericImageView;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_persist::PersistInstance;
use sqlx::{prelude::FromRow, Executor, PgPool};
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

#[derive(Deserialize)]
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

#[get("/13/sql")]
async fn day_13_select(data: web::Data<AppState>) -> impl Responder {
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(20231213_i64)
        .fetch_one(&data.pool)
        .await
        .expect("do sql");

    row.0.to_string()
}

#[post("/13/reset")]
async fn day_13_reset(data: web::Data<AppState>) -> impl Responder {
    let _ = &data
        .pool
        .execute(include_str!("../schemas/day13schema.sql"))
        .await
        .expect("do sql");

    HttpResponse::Ok()
}

#[derive(Deserialize, FromRow)]
struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

#[post("/13/orders")]
async fn day_13_create_orders(
    orders: web::Json<Vec<Order>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let orders = orders.into_inner();

    for order in orders {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name)
        .bind(order.quantity)
        .fetch_all(&data.pool)
        .await
        .expect("do sql");
    }

    HttpResponse::Ok()
}

#[get("/13/orders/total")]
async fn day_13_orders_total(data: web::Data<AppState>) -> Result<impl Responder> {
    let orders: Vec<Order> = sqlx::query_as::<_, Order>("SELECT * FROM orders")
        .fetch_all(&data.pool)
        .await
        .expect("do sql");

    let gift_sum: i32 = orders.iter().map(|order| order.quantity).sum();

    Ok(web::Json(json!({
        "total": gift_sum
    })))
}

#[get("/13/orders/popular")]
async fn day_13_popular(data: web::Data<AppState>) -> Result<impl Responder> {
    let orders: Vec<Order> = sqlx::query_as::<_, Order>("SELECT * FROM orders")
        .fetch_all(&data.pool)
        .await
        .expect("do sql");

    let mut gift_count: HashMap<String, i32> = HashMap::new();

    orders.iter().for_each(|gift| {
        let current_count = gift_count.get(&gift.gift_name).unwrap_or(&0);
        gift_count.insert(gift.gift_name.clone(), current_count + gift.quantity);
    });

    let mut popular = gift_count
        .iter()
        .map(|gift| (gift.0, gift.1))
        .collect::<Vec<_>>();

    popular.sort_by(|a, b| b.1.cmp(&a.1));

    let popular_name = match popular.get(0) {
        Some(gift) => Some(gift.0),
        None => None,
    };

    Ok(web::Json(json!({
        "popular": popular_name
    })))
}

#[derive(Deserialize)]
struct HtmlReq {
    content: String,
}

#[derive(Template)]
#[template(path = "day14.html", escape = "none")]
struct Day14UnsafeTemplate {
    content: String,
}

#[derive(Template)]
#[template(path = "day14.html")]
struct Day14SafeTemplate {
    content: String,
}

#[post("/14/unsafe")]
async fn day_14_unsafe(body: web::Json<HtmlReq>) -> impl Responder {
    let body = body.into_inner();
    let content = body.content;

    Day14UnsafeTemplate { content: content }
}

#[post("/14/safe")]
async fn day_14_safe(body: web::Json<HtmlReq>) -> impl Responder {
    let body = body.into_inner();
    let content = body.content;

    Day14SafeTemplate { content: content }
}

#[derive(Debug, Display, Error)]
enum NiceError {
    #[display(fmt = "naughty")]
    BadClientData,
}

#[derive(Debug, Display, Error)]
enum GameError {
    #[display(fmt = "8 chars")]
    Not8Chars,

    #[display(fmt = "more types of chars")]
    MoreCharTypes,

    #[display(fmt = "55555")]
    FiveFives,

    #[display(fmt = "math is hard")]
    MathIsHard,

    #[display(fmt = "not joyful enough")]
    NotJoyfulEnough,

    #[display(fmt = "illegal: no sandwich")]
    NoSandwich,

    #[display(fmt = "outranged")]
    Outranged,

    #[display(fmt = "😳")]
    ShockingEmoji,

    #[display(fmt = "not a coffee brewer")]
    NotACoffeeBrewer,
}

impl error::ResponseError for NiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
               "result": "naughty"
            }))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl error::ResponseError for GameError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
               "result": "naughty",
               "reason": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            GameError::NotJoyfulEnough => StatusCode::NOT_ACCEPTABLE,
            GameError::NoSandwich => StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            GameError::Outranged => StatusCode::RANGE_NOT_SATISFIABLE,
            GameError::ShockingEmoji => StatusCode::UPGRADE_REQUIRED,
            GameError::NotACoffeeBrewer => StatusCode::IM_A_TEAPOT,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Deserialize)]
struct PasswordReq {
    input: String,
}

#[post("/15/nice")]
async fn day_15_nice(body: web::Json<PasswordReq>) -> Result<impl Responder, NiceError> {
    let body = body.into_inner();
    let input = body.input;

    let vowels = String::from("aeiouy");
    let bad_sequences = Vec::from([
        String::from("ab"),
        String::from("cd"),
        String::from("pq"),
        String::from("xy"),
    ]);

    let vowel_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| vowels.contains(**c as char))
        .collect::<Vec<_>>()
        .len() as i32;
    let repeat_match = input
        .as_bytes()
        .windows(2)
        .any(|c| c[0] == c[1] && (c[0] as char).is_alphabetic());
    let bad_sequence_match = input
        .as_bytes()
        .windows(2)
        .any(|c| bad_sequences.contains(&String::from_utf8(c.to_vec()).unwrap()));

    if vowel_count < 3 || !repeat_match || bad_sequence_match {
        return Err(NiceError::BadClientData);
    }

    Ok(web::Json(json!({
        "result": "nice"
    })))
}

#[post("/15/game")]
async fn day_15_game(body: web::Json<PasswordReq>) -> Result<impl Responder, GameError> {
    let body = body.into_inner();
    let input = body.input;

    // rule 2
    let lowercase_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| (**c as char).is_lowercase())
        .collect::<Vec<_>>()
        .len() as i32;
    let uppercase_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| (**c as char).is_uppercase())
        .collect::<Vec<_>>()
        .len() as i32;

    // rule 2 & 3
    let digit_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| (**c as char).is_numeric())
        .collect::<Vec<_>>()
        .len() as i32;

    // rule 4
    let r4rx = Regex::new(r"([0-9]+)").unwrap();
    let year_sum: i32 = r4rx
        .captures_iter(&input)
        .map(|x| x.extract())
        .into_iter()
        .map(|(_, [num])| num.parse::<i32>().unwrap_or_default())
        .sum();

    // rule 5
    let r5rx = Regex::new(r"j+?.+?o+?.+?y+?").unwrap();
    let new_joy_match_count = match r5rx.captures(&input) {
        Some(matches) => matches.len(),
        None => 0,
    };

    // rule 6
    let sandwich_match = input
        .as_bytes()
        .windows(3)
        .any(|c| c[0] == c[2] && (c[0] as char).is_alphabetic());

    // rule 7
    let r7rx = Regex::new(r"[\u2980-\u2BFF]").unwrap();
    let range_match = r7rx.is_match(&input);

    // rule 8
    let r8rx = Regex::new(r"[\p{Emoji}--\p{Ascii}]").unwrap();
    let emoji_match = r8rx.is_match(&input);

    // rule 9
    let mut sha256sum = Sha256::new();
    sha256sum.update(input.as_bytes());
    let input_as_sha256 = sha256sum.finalize();
    let input_hex = hex::encode(input_as_sha256);

    // 1
    if input.len() < 8 {
        return Err(GameError::Not8Chars);
    }

    // 2
    if lowercase_count < 1 || uppercase_count < 1 || digit_count < 1 {
        return Err(GameError::MoreCharTypes);
    }

    // 3
    if digit_count < 5 {
        return Err(GameError::FiveFives);
    }

    // 4
    if year_sum != 2023i32 {
        return Err(GameError::MathIsHard);
    }

    // 5
    if !(new_joy_match_count == 1) {
        return Err(GameError::NotJoyfulEnough);
    }

    // 6
    if !sandwich_match {
        return Err(GameError::NoSandwich);
    }

    // 7
    if !range_match {
        return Err(GameError::Outranged);
    }

    // 8
    if !emoji_match {
        return Err(GameError::ShockingEmoji);
    }

    // 9
    if !input_hex.ends_with("a") {
        return Err(GameError::NotACoffeeBrewer);
    }

    Ok(web::Json(json!({
        "result": "nice",
        "reason": "that's a nice password"
    })))
}

#[post("/18/reset")]
async fn day_18_reset(data: web::Data<AppState>) -> impl Responder {
    let _ = &data
        .pool
        .execute(include_str!("../schemas/day18schema.sql"))
        .await
        .expect("do sql");

    HttpResponse::Ok()
}

#[post("/18/orders")]
async fn day_18_create_orders(
    orders: web::Json<Vec<Order>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let orders = orders.into_inner();

    for order in orders {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name)
        .bind(order.quantity)
        .fetch_all(&data.pool)
        .await
        .expect("do sql");
    }

    HttpResponse::Ok()
}

#[derive(Deserialize)]
struct Region {
    id: i32,
    name: String,
}

#[post("/18/regions")]
async fn day_18_create_regions(
    regions: web::Json<Vec<Region>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let regions = regions.into_inner();

    for region in regions {
        sqlx::query("INSERT INTO regions (id, name) VALUES ($1, $2)")
            .bind(region.id)
            .bind(region.name)
            .fetch_all(&data.pool)
            .await
            .expect("do sql");
    }

    HttpResponse::Ok()
}

#[derive(FromRow, Serialize)]
struct RegionTotalRes {
    region: String,
    total: i64,
}

#[get("/18/regions/total")]
async fn day_18_regions_total(data: web::Data<AppState>) -> Result<impl Responder> {
    // trash query
    let region_totals: Vec<RegionTotalRes> = sqlx::query_as::<_, RegionTotalRes>(
        "SELECT
	regions.id,
	regions.name AS region,
	COALESCE(O.total, 0) AS total
FROM
	regions
	LEFT JOIN (
		SELECT
			region_id,
			SUM(quantity) AS total
		FROM
			orders
		GROUP BY
			orders.region_id) O ON regions.id = O.region_id
			WHERE regions.id IN (SELECT region_id FROM orders)
",
    )
    .fetch_all(&data.pool)
    .await
    .expect("do sql");

    Ok(web::Json(json!(region_totals)))
}

#[derive(FromRow)]
struct RegionTopGiftsRow {
    name: String,
    gift_name: Option<String>,
}

#[derive(Serialize)]
struct RegionTopGiftsRes {
    region: String,
    top_gifts: Vec<String>,
}

#[get("/18/regions/top_list/{max_list}")]
async fn day_18_top_list(
    max_list: web::Path<usize>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let max_list = max_list.into_inner();

    // trash query
    let region_top_gifts: Vec<RegionTopGiftsRow> = sqlx::query_as::<_, RegionTopGiftsRow>(
        "SELECT
	*
FROM
	regions
	LEFT JOIN (
		SELECT
			region_id,
			gift_name,
			SUM(quantity)
		FROM
			orders
		WHERE quantity < 10000
		GROUP BY
			region_id,
			gift_name) orders ON regions.id = orders.region_id
GROUP BY
	regions.id,
	orders.region_id,
	orders.gift_name,
	orders.sum
ORDER BY
	regions.name ASC,
	orders.sum DESC,
	orders.gift_name ASC",
    )
    .fetch_all(&data.pool)
    .await
    .expect("do sql");

    let mut gift_map: HashMap<String, Vec<Option<String>>> = HashMap::new();

    region_top_gifts
        .iter()
        .for_each(|entry| match gift_map.get_mut(&entry.name) {
            Some(gift_list) => {
                if gift_list.len() < max_list {
                    gift_list.push(entry.gift_name.clone());
                }
            }
            None => {
                let first_gift = entry.gift_name.clone();
                gift_map.insert(entry.name.clone(), vec![first_gift]);
            }
        });

    let mut region_top_gifts = gift_map
        .iter()
        .map(|(region_name, top_gifts)| RegionTopGiftsRes {
            region: region_name.clone(),
            top_gifts: top_gifts
                .clone()
                .into_iter()
                .filter_map(|x| x)
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    // sql shows its sorted but still need to do this...
    region_top_gifts.sort_by(|a, b| a.region.cmp(&b.region));

    Ok(web::Json(json!(region_top_gifts)))
}

struct AppState {
    persist: PersistInstance,
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_persist::Persist] persist: PersistInstance,
    #[shuttle_shared_db::Postgres] pool: PgPool,
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
        cfg.service(day_13_select);
        cfg.service(day_13_reset);
        cfg.service(day_13_create_orders);
        cfg.service(day_13_orders_total);
        cfg.service(day_13_popular);
        cfg.service(day_14_unsafe);
        cfg.service(day_14_safe);
        cfg.service(day_15_nice);
        cfg.service(day_15_game);
        cfg.service(day_18_reset);
        cfg.service(day_18_create_orders);
        cfg.service(day_18_create_regions);
        cfg.service(day_18_regions_total);
        cfg.service(day_18_top_list);

        // 32MB
        cfg.app_data(web::PayloadConfig::new(1 << 25));

        let app_data = web::Data::new(AppState { persist, pool });
        cfg.app_data(app_data.clone());
    };

    Ok(config.into())
}
