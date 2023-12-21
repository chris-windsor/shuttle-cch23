use actix_web::{post, web, HttpResponse, Responder, Result};
use serde::Deserialize;
use serde_json::json;

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
pub async fn day_4_strength(reindeer: web::Json<Vec<Reindeer>>) -> impl Responder {
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
pub async fn day_4_contest(reindeer: web::Json<Vec<Reindeer>>) -> Result<impl Responder> {
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
