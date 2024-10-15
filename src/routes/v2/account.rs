use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::core::locale::{Language, LANGUAGES};
use crate::core::object::UserData;
use crate::core::state::AppState;
use crate::core::system_core::AnyResponder;
use crate::global::DATABASE;

#[get("/account")]
pub async fn account_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de/account"),
        Some(Language::English) => Redirect::to("/v2/en/account"),
        _ => Redirect::to("/v2/en/account"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>/account", rank = 3)]
pub async fn account(lang: &str, state: &State<AppState>, jar: &CookieJar<'_>) -> AnyResponder {
    if !LANGUAGES.contains(&lang) {
        return AnyResponder::Template(Template::render("404", context! {
            uri: format!("/v2/{lang}/login")
        }));
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

    AnyResponder::Template(Template::render("account", context! {
        title: &strings.account_settings,
        strings: &strings,
        lang: &lang,
        is_authenticated: is_authenticated,
        user: user,
    }))
}