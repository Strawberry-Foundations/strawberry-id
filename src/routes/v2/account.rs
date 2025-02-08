use crate::core::locale::{Language, LANGUAGES};
use crate::core::object::UserData;
use crate::core::state::AppState;
use crate::core::system_core::AnyResponder;
use crate::core::totp::StrawberryIdTotp;
use crate::global::DATABASE;

use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::content::RawJson;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::State;
use rocket_dyn_templates::{context, Template};

#[derive(FromForm)]
pub struct TotpForm {
    pub totp_code: String,
}

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
        return AnyResponder::Template(Template::render(
            "404",
            context! {
                uri: format!("/v2/{lang}/login")
            },
        ));
    };

    let strings = state.messages.get(lang).cloned().unwrap();
    let is_authenticated: bool;

    let user = match jar.get_private("_strawberryid.username") {
        Some(res) => {
            is_authenticated = true;
            DATABASE.get_user_data(res.value().to_string()).await
        }
        None => {
            is_authenticated = false;
            UserData::default()
        }
    };

    AnyResponder::Template(Template::render(
        "account",
        context! {
            title: &strings.account_settings,
            strings: &strings,
            lang: &lang,
            is_authenticated: is_authenticated,
            user: user,
            action: "none",
        },
    ))
}

#[get("/<_lang>/account/totp/qrcode")]
pub async fn generate_qr_code(_lang: &str, jar: &CookieJar<'_>) -> RawJson<String> {
    let user = jar
        .get_private("_strawberryid.username")
        .unwrap()
        .value()
        .to_string();

    let totp = StrawberryIdTotp::setup(&user);

    DATABASE
        .save_totp_secret(user.clone(), totp.secret)
        .await
        .unwrap();

    let json = json!({
        "qr_code": totp.qr_code,
        "secret": totp.secret_base32,
    });

    let json_string = serde_json::to_string_pretty(&json).unwrap();

    RawJson(json_string)
}

#[post("/<lang>/account", data = "<totp_form>")]
pub async fn setup_totp(lang: &str, totp_form: Form<TotpForm>, state: &State<AppState>, jar: &CookieJar<'_>) -> AnyResponder {
    let totp_code = &totp_form.totp_code;

    if !LANGUAGES.contains(&lang) {
        return AnyResponder::Template(Template::render(
            "404",
            context! {
                uri: format!("/v2/{lang}/login")
            },
        ));
    };

    let strings = state.messages.get(lang).cloned().unwrap();
    let is_authenticated: bool;

    let user = match jar.get_private("_strawberryid.username") {
        Some(res) => {
            is_authenticated = true;
            DATABASE.get_user_data(res.value().to_string()).await
        }
        None => {
            is_authenticated = false;
            UserData::default()
        }
    };

    if !StrawberryIdTotp::check(&user.totp_secret.clone(), totp_code) {
        DATABASE
            .set_totp(user.username.clone(), "false".to_string())
            .await
            .unwrap();
        return AnyResponder::Template(Template::render(
            "account",
            context! {
                title: &strings.account_settings,
                strings: &strings,
                lang: &lang,
                is_authenticated: is_authenticated,
                user: user,
                action: "setup_totp_failed",
            },
        ));
    }

    DATABASE
        .set_totp(user.username.clone(), "true".to_string())
        .await
        .unwrap();

    AnyResponder::Template(Template::render(
        "account",
        context! {
            title: &strings.account_settings,
            strings: &strings,
            lang: &lang,
            is_authenticated: is_authenticated,
            user: user,
            action: "setup_totp_success",
        },
    ))
}
