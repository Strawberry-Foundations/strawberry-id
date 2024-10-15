use rocket::response::Redirect;
use crate::core::locale::Language;

#[get("/")]
pub async fn root_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de"),
        Some(Language::English) => Redirect::to("/v2/en"),
        _ => Redirect::to("/v2/en"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>", rank = 3)]
pub async fn root(lang: &str) -> Redirect {
    match lang {
        "de" => Redirect::to("/v2/de"),
        "en" => Redirect::to("/v2/en"),
        _ => Redirect::to("/v2/en"), // Default to English if language not specified or supported
    }
}