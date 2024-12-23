use rocket::http::CookieJar;
use rocket::State;
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};

use crate::core::state::AppState;
use crate::core::locale::{Language, LANGUAGES};
use crate::core::object::UserData;
use crate::global::DATABASE;

#[get("/")]
pub async fn index_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de"),
        Some(Language::English) => Redirect::to("/v2/en"),
        _ => Redirect::to("/v2/en"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>", rank = 3)]
pub async fn index(lang: &str, state: &State<AppState>, jar: &CookieJar<'_>) -> Template {
    if !LANGUAGES.contains(&lang) {
        return Template::render("404", context! {
            uri: format!("/v2/{lang}")
        });
    };

    let strings = state.messages.get(lang).cloned().unwrap();
    let is_authenticated: bool;

    let user = match jar.get_private("_strawberryid.username") {
        Some(res) => {
            is_authenticated = true;
            DATABASE.get_user_data(res.value().to_string()).await
        },
        None => {
            is_authenticated = false;
            UserData::default()
        }
    };

    Template::render("index", context! {
        title: &"!!empty",
        strings: &strings,
        lang: &lang,
        is_authenticated: is_authenticated,
        user: user
    })
}