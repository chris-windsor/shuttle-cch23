use actix_web::{post, web, Responder, Result};
use serde_json::json;

#[post("/6")]
pub async fn day_6(body: web::Bytes) -> Result<impl Responder> {
    let doc: Vec<_> = body
        .windows(3)
        .enumerate()
        .filter_map(|(_pos, chunk)| {
            if String::from_utf8(chunk.to_vec()).unwrap_or_default() == "elf" {
                return Some("elf");
            }
            None
        })
        .collect();

    let elf_count = doc.len();

    let (elf_on_shelf_count, shelf_no_elf_count) = body
        .windows(5)
        .enumerate()
        .filter_map(|(pos, chunk)| {
            if String::from_utf8(chunk.to_vec()).unwrap_or_default() == "shelf" {
                if (String::from_utf8(body.slice(..pos).to_vec()))
                    .unwrap_or_default()
                    .ends_with("elf on a ")
                {
                    return Some(&true);
                } else {
                    return Some(&false);
                }
            }
            None
        })
        .fold((0, 0), |(yay, nay), cur| {
            // woah. cool
            (yay + (cur == &true) as i32, nay + (cur == &false) as i32)
        });

    Ok(web::Json(json!({
        "elf": elf_count,
        "elf on a shelf": elf_on_shelf_count,
        "shelf with no elf on it": shelf_no_elf_count
    })))
}
