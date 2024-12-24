use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::RwLock;
use lazy_static::lazy_static;

use crate::core::locale::{Language, LANGUAGES, Messages};
use crate::core::object::{CodeType, LoginForm, LoginMeta, UserData};
use crate::core::params::LoginParams;
use crate::core::state::AppState;
use crate::core::system_core::{AnyResponder, verify_password};
use crate::global::{CORE, DATABASE};


lazy_static! {
    pub static ref TEMP_STORAGE: RwLock<HashMap<String, UserData>> = RwLock::new(HashMap::new());
}

pub async fn template_responder(
    strings: &Messages, lang: &str, params: &LoginParams, meta: &LoginMeta,
    full_params: &str, redirect_after_login: &str, service_after_login: &str
) -> AnyResponder {
    AnyResponder::Template(Template::render("login", context! {
        title: &strings.login,
        strings: strings,
        lang: lang,
        meta: meta,
        params: params,
        redirect_after_login: redirect_after_login,
        service_after_login: service_after_login,
        full_params: full_params
    }))
}

#[get("/login")]
pub async fn login_no_lang(lang: Option<Language>) -> Redirect {
    match lang {
        Some(Language::German) => Redirect::to("/v2/de/login"),
        Some(Language::English) => Redirect::to("/v2/en/login"),
        _ => Redirect::to("/v2/en/login"), // Default to English if language not specified or supported
    }
}

#[get("/<lang>/login?<params..>", rank = 3)]
pub async fn login(lang: &str, state: &State<AppState>, mut params: LoginParams, uri: &rocket::http::uri::Origin<'_>) -> Template {
    if !LANGUAGES.contains(&lang) {
        return Template::render("404", context! {
            uri: format!("/v2/{lang}/login")
        });
    };

    let meta = LoginMeta::collect(&mut params, lang, state);
    let strings = state.messages.get(lang).cloned().unwrap();

    let redirect_after_login = strings.redirect_after_login.replace("%s", &params.redirect);
    let service_after_login = strings.service_after_login.replace("%s", &meta.service_name);

    let full_params = match uri.query() {
        Some(query) => query.to_string(),
        None => String::new()
    };

    Template::render("login", context! {
        title: &strings.login,
        strings: &strings,
        lang: &lang,
        meta: &meta,
        params: &params,
        redirect_after_login: &redirect_after_login,
        service_after_login: &service_after_login,
        full_params: &full_params
    })
}

#[post("/<lang>/login?<params..>", data = "<form>")]
pub async fn login_post(
    lang: &str, form: Form<LoginForm>, state: &State<AppState>,
    mut params: LoginParams, jar: &CookieJar<'_>, uri: &rocket::http::uri::Origin<'_>
) -> AnyResponder {
    let mut meta = LoginMeta::collect(&mut params, lang, state);

    let username = &form.username;
    let password = &form.password;

    let strings = state.messages.get(lang).cloned().unwrap();

    let redirect_after_login = strings.redirect_after_login.replace("%s", &params.redirect);
    let service_after_login = strings.service_after_login.replace("%s", &meta.service_name);

    let full_params = match uri.query() {
        Some(query) => query.to_string(),
        None => String::new()
    };

    let user_base = match DATABASE.get_password(username.to_string()).await {
        Some(res) => res,
        None => {
            meta.info_message = strings.wrong_username.clone();
            meta.error = true;
            return template_responder(&strings, lang, &params, &meta, &full_params, &redirect_after_login, &service_after_login).await
        }
    };

    if verify_password(user_base.password, password) {
        let user_data = DATABASE.get_user_data(user_base.username).await;
        let data = user_data.clone();

        if data.account_enabled == "false" {
            meta.info_message = strings.account_disabled.clone();
            meta.error = true;
            return template_responder(&strings, lang, &params, &meta, &full_params, &redirect_after_login, &service_after_login).await
        };

        if data.totp_enabled == "true" {
            meta.totp_required = true;
            let temp_token = Uuid::new_v4().to_string();
            jar.add_private(rocket::http::Cookie::new("_strawberryid.temp_token", temp_token.clone()));
            TEMP_STORAGE.write().await.insert(temp_token, data);
            return AnyResponder::Template(Template::render("login", context! {
                title: &strings.login,
                strings: &strings,
                lang: lang,
                meta: meta,
                params: params,
                redirect_after_login: redirect_after_login,
                service_after_login: service_after_login,
                full_params: full_params
            }))
        }

        let code = CORE.write().await.generate_code(user_data, CodeType::Website);

        jar.add_private(("_strawberryid.username", data.username.clone()));
        jar.add_private(("_strawberryid.email", data.email.clone()));
        jar.add_private(("_strawberryid.full_name", data.full_name.clone()));
        jar.add_private(("_strawberryid.profile_picture_url", data.profile_picture_url.clone()));

        if meta.code != 0 {
            return AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                    "/v2/{}/login/oauth_dialog/{}?code={}",
                    meta.language,
                    params.service,
                    meta.code
                ))
            ))
        }

        if meta.redirect_after_login {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                        "https://{}/callback?hl={}&code={}",
                        params.redirect,
                        params.hl,
                        code
                    ))
            ))
        }
        else if params.oauth && !meta.service_after_login {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                    "/v2/{}/login/oauth/{}",
                    meta.language,
                    params.service
                ))
            ))
        }
        else {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!("/v2/{}/account", meta.language))
            ))
        }
    }
    else {
        meta.info_message = strings.wrong_username_or_password.clone();
        meta.error = true;
        template_responder(&strings, lang, &params, &meta, &full_params, &redirect_after_login, &service_after_login).await
    }
}