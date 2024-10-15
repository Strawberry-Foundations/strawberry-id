use rocket::response::Redirect;
use crate::core::locale::Language;

#[get("/")]
pub async fn v1_index_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de/login"),
        Some(Language::English) => Redirect::to("/v2/en/login"),
        _ => Redirect::to("/v2/en"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>", rank = 3)]
pub async fn v1_index(lang: &str) -> Redirect {
    match lang {
        "de" => Redirect::to("/v2/de/login"),
        "en" => Redirect::to("/v2/en/login"),
        _ => Redirect::to("/v2/en/login"), // Default to English if language not specified or supported
    }
}