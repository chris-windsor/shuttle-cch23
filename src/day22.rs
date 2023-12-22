use std::{collections::HashSet, iter};

use actix_web::{post, web, HttpResponse, Responder};

#[post("/22/integers")]
pub async fn day_22_integers(integers: web::Bytes) -> impl Responder {
    let integers = String::from_utf8(integers.to_vec()).expect("parse body");
    let mut matches: HashSet<&str> = HashSet::new();
    integers.lines().for_each(|line| {
        if !matches.remove(line) {
            matches.insert(line);
        }
    });
    let present_count = matches.iter().nth(0).unwrap().parse::<usize>().unwrap();
    let present_string = "🎁".repeat(present_count);
    HttpResponse::Ok().body(present_string)
}
