use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    post, web, HttpResponse, Responder,
};
use derive_more::{Display, Error};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Debug, Display, Error)]
enum NiceError {
    #[display(fmt = "naughty")]
    BadClientData,
}

#[derive(Debug, Display, Error)]
enum GameError {
    #[display(fmt = "8 chars")]
    Not8Chars,

    #[display(fmt = "more types of chars")]
    MoreCharTypes,

    #[display(fmt = "55555")]
    FiveFives,

    #[display(fmt = "math is hard")]
    MathIsHard,

    #[display(fmt = "not joyful enough")]
    NotJoyfulEnough,

    #[display(fmt = "illegal: no sandwich")]
    NoSandwich,

    #[display(fmt = "outranged")]
    Outranged,

    #[display(fmt = "😳")]
    ShockingEmoji,

    #[display(fmt = "not a coffee brewer")]
    NotACoffeeBrewer,
}

impl error::ResponseError for NiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
               "result": "naughty"
            }))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl error::ResponseError for GameError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
               "result": "naughty",
               "reason": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            GameError::NotJoyfulEnough => StatusCode::NOT_ACCEPTABLE,
            GameError::NoSandwich => StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            GameError::Outranged => StatusCode::RANGE_NOT_SATISFIABLE,
            GameError::ShockingEmoji => StatusCode::UPGRADE_REQUIRED,
            GameError::NotACoffeeBrewer => StatusCode::IM_A_TEAPOT,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Deserialize)]
struct PasswordReq {
    input: String,
}

#[post("/15/nice")]
pub async fn day_15_nice(body: web::Json<PasswordReq>) -> Result<impl Responder, NiceError> {
    let body = body.into_inner();
    let input = body.input;

    let vowels = String::from("aeiouy");
    let bad_sequences = Vec::from([
        String::from("ab"),
        String::from("cd"),
        String::from("pq"),
        String::from("xy"),
    ]);

    let vowel_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| vowels.contains(**c as char))
        .collect::<Vec<_>>()
        .len() as i32;
    let repeat_match = input
        .as_bytes()
        .windows(2)
        .any(|c| c[0] == c[1] && (c[0] as char).is_alphabetic());
    let bad_sequence_match = input
        .as_bytes()
        .windows(2)
        .any(|c| bad_sequences.contains(&String::from_utf8(c.to_vec()).unwrap()));

    if vowel_count < 3 || !repeat_match || bad_sequence_match {
        return Err(NiceError::BadClientData);
    }

    Ok(web::Json(json!({
        "result": "nice"
    })))
}

#[post("/15/game")]
async fn day_15_game(body: web::Json<PasswordReq>) -> Result<impl Responder, GameError> {
    let body = body.into_inner();
    let input = body.input;

    // rule 2
    let lowercase_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| (**c as char).is_lowercase())
        .collect::<Vec<_>>()
        .len() as i32;
    let uppercase_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| (**c as char).is_uppercase())
        .collect::<Vec<_>>()
        .len() as i32;

    // rule 2 & 3
    let digit_count: i32 = input
        .as_bytes()
        .iter()
        .filter(|c| (**c as char).is_numeric())
        .collect::<Vec<_>>()
        .len() as i32;

    // rule 4
    let r4rx = Regex::new(r"([0-9]+)").unwrap();
    let year_sum: i32 = r4rx
        .captures_iter(&input)
        .map(|x| x.extract())
        .into_iter()
        .map(|(_, [num])| num.parse::<i32>().unwrap_or_default())
        .sum();

    // rule 5
    let r5rx = Regex::new(r"j+?.+?o+?.+?y+?").unwrap();
    let new_joy_match_count = match r5rx.captures(&input) {
        Some(matches) => matches.len(),
        None => 0,
    };

    // rule 6
    let sandwich_match = input
        .as_bytes()
        .windows(3)
        .any(|c| c[0] == c[2] && (c[0] as char).is_alphabetic());

    // rule 7
    let r7rx = Regex::new(r"[\u2980-\u2BFF]").unwrap();
    let range_match = r7rx.is_match(&input);

    // rule 8
    let r8rx = Regex::new(r"[\p{Emoji}--\p{Ascii}]").unwrap();
    let emoji_match = r8rx.is_match(&input);

    // rule 9
    let mut sha256sum = Sha256::new();
    sha256sum.update(input.as_bytes());
    let input_as_sha256 = sha256sum.finalize();
    let input_hex = hex::encode(input_as_sha256);

    // 1
    if input.len() < 8 {
        return Err(GameError::Not8Chars);
    }

    // 2
    if lowercase_count < 1 || uppercase_count < 1 || digit_count < 1 {
        return Err(GameError::MoreCharTypes);
    }

    // 3
    if digit_count < 5 {
        return Err(GameError::FiveFives);
    }

    // 4
    if year_sum != 2023i32 {
        return Err(GameError::MathIsHard);
    }

    // 5
    if !(new_joy_match_count == 1) {
        return Err(GameError::NotJoyfulEnough);
    }

    // 6
    if !sandwich_match {
        return Err(GameError::NoSandwich);
    }

    // 7
    if !range_match {
        return Err(GameError::Outranged);
    }

    // 8
    if !emoji_match {
        return Err(GameError::ShockingEmoji);
    }

    // 9
    if !input_hex.ends_with("a") {
        return Err(GameError::NotACoffeeBrewer);
    }

    Ok(web::Json(json!({
        "result": "nice",
        "reason": "that's a nice password"
    })))
}
