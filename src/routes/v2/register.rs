use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::State;

use crate::core::locale::Language;
use crate::core::state::AppState;

#[get("/register")]
pub async fn register_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de/register"),
        Some(Language::English) => Redirect::to("/v2/en/register"),
        _ => Redirect::to("/v2/en/register"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>/register", rank = 3)]
pub async fn register(lang: &str, _state: &State<AppState>) -> RawHtml<String> {
    /* if !LANGUAGES.contains(&lang) {
        return Template::render("404", context! {
            uri: format!("/v2/{lang}/login")
        });
    }; */

    RawHtml(format!("[{lang}] Currently not supported"))
}