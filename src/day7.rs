use std::collections::HashMap;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder, Result};
use base64::Engine;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde_json::json;

fn decode_cookie_header(cookie_header: &HeaderValue) -> String {
    let header_value = cookie_header.to_str().unwrap();
    let recipe_encoded = &header_value["recipe=".len()..];

    String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(recipe_encoded)
            .unwrap(),
    )
    .unwrap()
}

#[get("/7/decode")]
pub async fn day_7_decode(req: HttpRequest) -> impl Responder {
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
pub async fn day_7_bake(req: HttpRequest) -> Result<impl Responder> {
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
