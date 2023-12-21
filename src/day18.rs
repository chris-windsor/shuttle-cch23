use std::collections::HashMap;

use actix_web::{get, post, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Executor, FromRow};

use crate::AppState;

#[post("/18/reset")]
pub async fn day_18_reset(data: web::Data<AppState>) -> impl Responder {
    let _ = &data
        .pool
        .execute(include_str!("../schemas/day18schema.sql"))
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
pub async fn day_18_regions_total(data: web::Data<AppState>) -> Result<impl Responder> {
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
