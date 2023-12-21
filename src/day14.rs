use actix_web::{post, web, Responder};
use askama::Template;
use serde::Deserialize;

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
pub async fn day_14_unsafe(body: web::Json<HtmlReq>) -> impl Responder {
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
