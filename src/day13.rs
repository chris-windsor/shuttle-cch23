use std::collections::HashMap;

use actix_web::{get, post, web, HttpResponse, Responder, Result};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Executor, FromRow};

use crate::AppState;

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
