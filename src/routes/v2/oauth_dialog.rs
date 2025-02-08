use crate::core::locale::LANGUAGES;
use crate::core::object::OAuthMeta;
use crate::core::params::ApiParams;
use crate::core::state::AppState;
use crate::core::system_core::AnyResponder;
use crate::global::{CONFIG, CORE, DATABASE};
use crate::utilities::name_parser;

use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

#[get("/<lang>/login/oauth_dialog/<service>?<params..>")]
pub async fn oauth_permit_dialog(
    lang: &str,
    service: &str,
    params: ApiParams,
    state: &State<AppState>,
    jar: &CookieJar<'_>,
) -> AnyResponder {
    let code = params.code;

    if !LANGUAGES.contains(&lang) {
        return AnyResponder::Template(Template::render(
            "404",
            context! {
                uri: format!("/v2/{lang}/login")
            },
        ));
    };

    let mut meta = OAuthMeta {
        service: service.to_string(),
        service_name: name_parser(state, service),
        trusted: CONFIG.vars.trusted_services.contains(&service.to_string()),
        ..Default::default()
    };

    match jar.get_private("_strawberryid.username") {
        Some(res) => res,
        None => {
            return AnyResponder::Redirect(Box::from(Redirect::to(format!(
                "/v2/{lang}/login?service={}&oauth=true&code={code}",
                meta.service
            ))))
        }
    };

    let strings = state.messages.get(lang).cloned().unwrap();

    if !CORE.read().await.oauth_codes.contains_key(&code) {
        meta.error = true;
        meta.info_message = strings.error_invalid_code.clone();
    }

    let service_after_login = strings.service_after_login.replace("%s", meta.service_name.as_str());
    
    let service_wants_to_login = strings
        .service_wants_to_login
        .replace("%1", meta.service_name.as_str())
        .replace("%2", format!("<u>{}</u>", code).as_str());

    AnyResponder::Template(Template::render(
        "oauth/dialog",
        context! {
            title: &strings.login,
            lang: &lang,
            strings: &strings,
            meta: &meta,
            service_after_login: &service_after_login,
            service_wants_to_login: &service_wants_to_login,
            code: &code,
        },
    ))
}

#[post("/<lang>/login/oauth_dialog/<service>?<params..>")]
pub async fn oauth_permit_dialog_post(
    lang: &str,
    service: &str,
    params: ApiParams,
    state: &State<AppState>,
    jar: &CookieJar<'_>,
) -> AnyResponder {
    let code = params.code;

    if !LANGUAGES.contains(&lang) {
        return AnyResponder::Template(Template::render(
            "404",
            context! {
                uri: format!("/v2/{lang}/login")
            },
        ));
    };

    let mut meta = OAuthMeta {
        service: service.to_string(),
        service_name: name_parser(state, service),
        trusted: CONFIG.vars.trusted_services.contains(&service.to_string()),
        ..Default::default()
    };

    let username = match jar.get_private("_strawberryid.username") {
        Some(res) => res,
        None => {
            return AnyResponder::Redirect(Box::from(Redirect::to(format!(
                "/v2/{lang}/login?service={}&oauth=true",
                meta.service
            ))))
        }
    };

    let strings = state.messages.get(lang).cloned().unwrap();

    if let Some(value) = CORE.write().await.oauth_codes.get_mut(&code) {
        if let Some(mut first_value) = value.first().cloned() {
            let user_data = DATABASE.get_user_data(username.value().to_string()).await;

            first_value.1 = user_data;

            *value = vec![(
                username.value().to_string(),
                first_value.1,
                first_value.2,
                first_value.3,
            )];
        }
        meta.info_message = strings.code_valid.clone()
    } else {
        meta.error = true;
        meta.info_message = strings.error_invalid_code.clone();
    }

    let login_success_subtitle = strings
        .login_success_subtitle
        .replace("%s", meta.service_name.as_str());

    AnyResponder::Template(Template::render(
        "oauth/success",
        context! {
            title: &strings.login,
            lang: &lang,
            strings: &strings,
            meta: &meta,
            code: &code,
            login_success_subtitle: &login_success_subtitle
        },
    ))
}
