use std::sync::{atomic::AtomicUsize, Arc};

use actix::Actor;
use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use day19::ChatServer;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_persist::PersistInstance;
use sqlx::PgPool;

mod day1;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day18;
mod day19;
mod day4;
mod day6;
mod day7;
mod day8;

#[get("/")]
async fn base() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/-1/error")]
async fn fake_error() -> impl Responder {
    HttpResponse::InternalServerError()
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
        cfg.service(day1::day_1);
        cfg.service(day4::day_4_strength);
        cfg.service(day4::day_4_contest);
        cfg.service(day6::day_6);
        cfg.service(day7::day_7_decode);
        cfg.service(day7::day_7_bake);
        cfg.service(day8::day_8_weight);
        cfg.service(day8::day_8_drop);
        cfg.service(day11::day_11_image);
        cfg.service(day11::day_11_red_pixels);
        cfg.service(day12::day_12_save);
        cfg.service(day12::day_12_load);
        cfg.service(day12::day_12_ulids);
        cfg.service(day12::day_12_lsb);
        cfg.service(day13::day_13_select);
        cfg.service(day13::day_13_reset);
        cfg.service(day13::day_13_create_orders);
        cfg.service(day13::day_13_orders_total);
        cfg.service(day13::day_13_popular);
        cfg.service(day14::day_14_unsafe);
        cfg.service(day14::day_14_safe);
        cfg.service(day15::day_15_nice);
        cfg.service(day15::day_15_game);
        cfg.service(day18::day_18_reset);
        cfg.service(day18::day_18_create_orders);
        cfg.service(day18::day_18_create_regions);
        cfg.service(day18::day_18_regions_total);
        cfg.service(day18::day_18_top_list);
        cfg.service(day19::day_19_ws);
        cfg.service(day19::day_19_reset);
        cfg.service(day19::day_19_views);
        cfg.service(day19::day_19_chat);

        // 32MB
        cfg.app_data(web::PayloadConfig::new(1 << 25));

        let app_data = web::Data::new(AppState { persist, pool });
        cfg.app_data(app_data.clone());

        let ws_19_state = Arc::new(AtomicUsize::new(0));
        let ws_19_server = ChatServer::new(ws_19_state.clone()).start();
        cfg.app_data(web::Data::from(ws_19_state.clone()));
        cfg.app_data(web::Data::new(ws_19_server.clone()));
    };

    Ok(config.into())
}
