use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

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
pub async fn day_8_weight(path: web::Path<u32>) -> impl Responder {
    let pokedex_number = path.into_inner();

    let pokemon = get_pokemon_by_id(pokedex_number).await;

    HttpResponse::Ok().body((pokemon.weight / 10.0).to_string())
}

#[get("/8/drop/{pokedex_number}")]
pub async fn day_8_drop(path: web::Path<u32>) -> impl Responder {
    let pokedex_number = path.into_inner();

    let pokemon = get_pokemon_by_id(pokedex_number).await;

    let time = f32::sqrt(2.0 * 10.0 / 9.825);
    let velocity = 9.825 * time;
    let momentum: f32 = (pokemon.weight as f32) * velocity / 10.0;

    HttpResponse::Ok().body(momentum.to_string())
}
