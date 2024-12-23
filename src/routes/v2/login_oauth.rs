use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::core::locale::{Language, LANGUAGES};
use crate::core::object::{OAuthForm, OAuthMeta};
use crate::core::state::AppState;
use crate::core::system_core::AnyResponder;
use crate::utilities::name_parser;
use crate::global::{CONFIG, CORE, DATABASE};

#[get("/login/oauth")]
pub async fn login_oauth_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de/login"),
        Some(Language::English) => Redirect::to("/v2/en/login"),
        _ => Redirect::to("/v2/en/login"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>/login/oauth")]
pub async fn login_oauth_lang_redir(lang: &str) -> Redirect {
    match lang {
        "de" => Redirect::to("/v2/de/login"),
        "en" => Redirect::to("/v2/en/login"),
        _ => Redirect::to("/v2/en/login"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>/login/oauth/<service>", rank = 3)]
pub async fn login_oauth(lang: &str, state: &State<AppState>, service: &str, jar: &CookieJar<'_>) -> AnyResponder {
    let mut meta = OAuthMeta {
        animation: String::from("slide-in-login"),
        ..Default::default()
    };

    if !LANGUAGES.contains(&lang) {
        return AnyResponder::Template(Template::render("404", context! {
            uri: format!("/v2/{lang}/login")
        }));
    };

    meta.service = service.to_string();

    meta.service_name = name_parser(state, &meta.service);
    meta.trusted = CONFIG.vars.trusted_services.contains(&meta.service);

    let strings = state.messages.get(lang).cloned().unwrap();

    let service_after_login = strings.service_after_login.replace("%s", meta.service_name.as_str());


    match jar.get_private("_strawberryid.username") {
        Some(_) => { },
        None => return AnyResponder::Redirect(Box::from(Redirect::to(format!("/v2/{lang}/login?service={}&oauth=true", meta.service))))
    }

    AnyResponder::Template(Template::render("oauth/login", context! {
        title: &strings.login,
        strings: &strings,
        lang: &lang,
        meta: &meta,
        service_after_login: &service_after_login,
    }))
}


#[post("/<lang>/login/oauth/<service>", data = "<form>")]
pub async fn login_oauth_post(lang: &str, service: &str, form: Form<OAuthForm>, state: &State<AppState>, jar: &CookieJar<'_>) -> AnyResponder {
    let mut meta = OAuthMeta {
        animation: String::from("none"),
        ..Default::default()
    };

    if !LANGUAGES.contains(&lang) {
        return AnyResponder::Template(Template::render("404", context! {
            uri: format!("/v2/{lang}/login")
        }));
    };

    meta.service = service.to_string();

    meta.service_name = name_parser(state, &meta.service);
    meta.trusted = CONFIG.vars.trusted_services.contains(&meta.service);


    let code = form.code;

    let strings = state.messages.get(lang).cloned().unwrap();
    let strings_clone = strings.clone();

    let service_after_login = strings.service_after_login.replace("%s", meta.service_name.as_str());

    if code == 0 || code.to_string().len() != 10 {
        meta.error = true;
        meta.info_message = strings.error_invalid_code;

        AnyResponder::Template(Template::render("oauth/login", context! {
            title: &strings.login,
            strings: &strings_clone,
            lang: &lang,
            meta: &meta,
            service_after_login: &service_after_login,
        }))
    }
    else {
        if let Some(value) = CORE.write().await.oauth_codes.get_mut(&code) {
            if let Some(mut first_value) = value.first().cloned() {
                let username = match jar.get_private("_strawberryid.username") {
                    Some(res) => res,
                    None => return AnyResponder::Redirect(Box::from(Redirect::to(
                        format!("/v2/{lang}/login?service={}&oauth=true", meta.service)
                        )))
                };

                let user_data = DATABASE.get_user_data(username.value().to_string()).await;

                first_value.1 = user_data;

                *value = vec![(username.value().to_string(), first_value.1, first_value.2, first_value.3)];
            }
            meta.info_message = strings.code_valid

        } else {
            meta.error = true;
            meta.info_message = strings.error_invalid_code;
        }

        AnyResponder::Template(Template::render("oauth/login", context! {
            title: &strings.login,
            strings: &strings_clone,
            lang: &lang,
            meta: &meta,
            service_after_login: &service_after_login,
        }))
    }
}
