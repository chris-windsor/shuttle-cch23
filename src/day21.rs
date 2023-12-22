use actix_web::{get, web, HttpResponse, Responder};
use s2::cellid::CellID;
use serde_json::Value;

use crate::AppState;

fn coordinates_to_dms(lat: f64, lon: f64, precision: u32) -> String {
    let power_float: f64 = 10u32.pow(precision).try_into().unwrap();

    let lat_d = lat.abs().trunc();
    let lon_d = lon.abs().trunc();

    let lat_d_d = (lat - lat.trunc()).abs();
    let lon_d_d = (lon - lon.trunc()).abs();

    let lat_m = (60.0 * lat_d_d).trunc();
    let lon_m = (60.0 * lon_d_d).trunc();

    let lat_s = ((3600.0 * lat_d_d - 60.0 * lat_m) * power_float)
        .round()
        .trunc()
        / power_float;
    let lon_s = ((3600.0 * lon_d_d - 60.0 * lon_m) * power_float)
        .round()
        .trunc()
        / power_float;

    let lat_dir = if lat > 0.0 { "N" } else { "S" };
    let lon_dir = if lon > 0.0 { "E" } else { "W" };

    format!("{lat_d}°{lat_m}'{lat_s}''{lat_dir} {lon_d}°{lon_m}'{lon_s}''{lon_dir}")
}

#[get("/21/coords/{bin}")]
pub async fn day_21_coords(bin: web::Path<String>) -> impl Responder {
    let bin = bin.into_inner();
    let bin: u64 = u64::from_str_radix(&bin, 2).unwrap();

    let s2_cell = s2::cell::Cell::from(CellID(bin));
    let dms = coordinates_to_dms(
        s2_cell.center().latitude().deg(),
        s2_cell.center().longitude().deg(),
        3,
    );

    HttpResponse::Ok().body(dms)
}

async fn coords_to_country_name(api_key: &String, coords: String) -> String {
    let position = reqwest::get(format!(
        "http://api.positionstack.com/v1/reverse?access_key={api_key}&query={coords}"
    ))
    .await
    .expect("make request");
    let position: Value = position.json().await.expect("parse json");

    // maybe better than deserializing from struct and traversing?
    let position_data = position.as_object().unwrap().get("data").unwrap();
    let positions_with_country = position_data
        .as_array()
        .unwrap()
        .into_iter()
        .filter(|entry| match entry.as_object().unwrap().get("country") {
            Some(country_val) => country_val != &Value::Null,
            None => false,
        })
        .collect::<Vec<_>>();
    let first_position = positions_with_country.get(0).unwrap().as_object().unwrap();
    let country_name = first_position.get("country").unwrap();
    let country_name = country_name.as_str().unwrap();

    country_name.to_string()
}

#[get("/21/country/{bin}")]
pub async fn day_21_country(bin: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let bin = bin.into_inner();
    let bin: u64 = u64::from_str_radix(&bin, 2).unwrap();

    let s2_cell = s2::cell::Cell::from(CellID(bin));
    let lat = s2_cell.center().latitude().deg();
    let lon = s2_cell.center().longitude().deg();
    let coords = format!("{lat},{lon}");

    let country = coords_to_country_name(&data.secrets.position_stack_api_key, coords).await;

    HttpResponse::Ok().body(country)
}
