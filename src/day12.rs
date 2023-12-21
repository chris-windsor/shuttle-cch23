use actix_web::{get, post, web, HttpResponse, Responder, Result};
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use serde_json::json;
use ulid::Ulid;
use uuid::Uuid;

use crate::AppState;

#[post("/12/save/{packet}")]
pub async fn day_12_save(packet: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let packet = packet.into_inner();

    let _ = data.persist.save::<NaiveTime>(&packet, Utc::now().time());

    HttpResponse::Ok()
}

#[get("/12/load/{packet}")]
pub async fn day_12_load(packet: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let packet = packet.into_inner();

    let current_time = Utc::now().time();
    let packet_time = data.persist.load::<NaiveTime>(&packet).unwrap();

    let diff = current_time - packet_time.clone();

    diff.num_seconds().to_string()
}

#[post("/12/ulids")]
pub async fn day_12_ulids(ulids: web::Json<Vec<Ulid>>) -> Result<impl Responder> {
    let uuids = ulids
        .0
        .iter()
        .map(|ulid| Uuid::from(ulid.clone()))
        .rev()
        .collect::<Vec<Uuid>>();

    Ok(web::Json(json!(uuids)))
}

#[post("/12/ulids/{weekday}")]
pub async fn day_12_lsb(
    weekday: web::Path<u8>,
    ulids: web::Json<Vec<Ulid>>,
) -> Result<impl Responder> {
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
