use crate::core::locale::{Language, Messages, LANGUAGES};
use crate::core::object::{CodeType, LoginForm, LoginMeta, UserData};
use crate::core::params::LoginParams;
use crate::core::state::AppState;
use crate::core::system_core::{verify_password, AnyResponder};
use crate::global::{CORE, DATABASE};

use lazy_static::lazy_static;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use rocket::serde::json::Json;
use rocket::serde::{Serialize};


// Neue Structs für JSON Responses
#[derive(Serialize)]
pub struct CheckUserResponse {
    user_exists: bool,
    user: Option<UserInfo>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct UserInfo {
    email: String,
    display_name: String,
    avatar: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    totp_required: Option<bool>,
    redirect_url: Option<String>,
    error: Option<String>,
}

#[derive(FromForm)]
pub struct CheckUserForm {
    username: String,
}

lazy_static! {
    pub static ref TEMP_STORAGE: RwLock<HashMap<String, UserData>> = RwLock::new(HashMap::new());
}

pub async fn template_responder(
    strings: &Messages,
    lang: &str,
    params: &LoginParams,
    meta: &LoginMeta,
    full_params: &str,
    redirect_after_login: &str,
    service_after_login: &str,
) -> AnyResponder {
    AnyResponder::Template(Template::render(
        "login",
        context! {
            title: &strings.login,
            strings: strings,
            lang: lang,
            meta: meta,
            params: params,
            redirect_after_login: redirect_after_login,
            service_after_login: service_after_login,
            full_params: full_params
        },
    ))
}

#[post("/<lang>/check-user", data = "<form>")]
pub async fn check_user_post(
    lang: &str,
    form: Form<CheckUserForm>,
    state: &State<AppState>,
) -> Json<CheckUserResponse> {
    let username = &form.username;
    
    // Prüfe ob User existiert
    let user_base = DATABASE.get_password(username.to_string()).await;
    
    if let Some(user_base) = user_base {
        let user_data = DATABASE.get_user_data(user_base.username).await;
        
        Json(CheckUserResponse {
            user_exists: true,
            user: Some(UserInfo {
                email: user_data.email.clone(),
                display_name: user_data.full_name.clone(),
                avatar: if user_data.profile_picture_url.is_empty() {
                    None
                } else {
                    Some(user_data.profile_picture_url.clone())
                },
            }),
            error: None,
        })
    } else {
        let strings = state.messages.get(lang).cloned().unwrap();
        Json(CheckUserResponse {
            user_exists: false,
            user: None,
            error: Some(strings.wrong_username.clone()),
        })
    }
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
        return Template::render(
            "404",
            context! {
                uri: format!("/v2/{lang}/login")
            },
        );
    };

    let meta = LoginMeta::collect(&mut params, lang, state);
    let strings = state.messages.get(lang).cloned().unwrap();

    let redirect_after_login = strings.redirect_after_login.replace("%s", &params.redirect);
    let service_after_login = strings.service_after_login.replace("%s", &meta.service_name);

    let full_params = match uri.query() {
        Some(query) => query.to_string(),
        None => String::new(),
    };

    Template::render(
        "login",
        context! {
            title: &strings.login,
            strings: &strings,
            lang: &lang,
            meta: &meta,
            params: &params,
            redirect_after_login: &redirect_after_login,
            service_after_login: &service_after_login,
            full_params: &full_params
        },
    )
}

#[post("/<lang>/login?<params..>", data = "<form>")]
pub async fn login_post(
    lang: &str,
    form: Form<LoginForm>,
    state: &State<AppState>,
    mut params: LoginParams,
    jar: &CookieJar<'_>,
    _uri: &rocket::http::uri::Origin<'_>,
) -> AnyResponder {    
    let meta = LoginMeta::collect(&mut params, lang, state);
    let username = &form.username;
    let password = &form.password;
    let strings = state.messages.get(lang).cloned().unwrap();

    let user_base = match DATABASE.get_password(username.to_string()).await {
        Some(res) => res,
        None => {
            // JSON Response für AJAX
            return AnyResponder::Json(Json(LoginResponse {
                success: false,
                totp_required: None,
                redirect_url: None,
                error: Some(strings.wrong_username.clone()),
            }));
        }
    };

    if verify_password(user_base.password, password) {
        let user_data = DATABASE.get_user_data(user_base.username).await;
        let data = user_data.clone();

        if data.account_enabled == "false" {
            return AnyResponder::Json(Json(LoginResponse {
                success: false,
                totp_required: None,
                redirect_url: None,
                error: Some(strings.account_disabled.clone()),
            }));
        }

        if data.totp_enabled == "true" {
            let temp_token = Uuid::new_v4().to_string();
            jar.add_private(rocket::http::Cookie::new(
                "_strawberryid.temp_token",
                temp_token.clone(),
            ));
            TEMP_STORAGE.write().await.insert(temp_token, data);
            
            return AnyResponder::Json(Json(LoginResponse {
                success: false,
                totp_required: Some(true),
                redirect_url: None,
                error: None,
            }));
        }

        let code = CORE
            .write()
            .await
            .generate_code(user_data, CodeType::Website);

        jar.add_private(("_strawberryid.username", data.username.clone()));
        jar.add_private(("_strawberryid.email", data.email.clone()));
        jar.add_private(("_strawberryid.full_name", data.full_name.clone()));
        jar.add_private(("_strawberryid.profile_picture_url", data.profile_picture_url.clone()));

        let redirect_url = if meta.code != 0 {
            format!("/v2/{}/login/oauth_dialog/{}?code={}", meta.language, params.service, meta.code)
        } else if meta.redirect_after_login && !params.redirect.is_empty() {
            let callback_url = urlencoding::decode(&params.redirect).unwrap_or_default().to_string();
            let lang_param = if !lang.is_empty() { lang } else { "en" };
            
            if callback_url.starts_with("http://") || callback_url.starts_with("https://") {
                format!("{}/callback?hl={}&code={}", callback_url, lang_param, code)
            } else {
                format!("https://{}/callback?hl={}&code={}", callback_url, lang_param, code)
            }
        } else if params.oauth && !params.service.is_empty() {
            format!("/v2/{}/login/oauth/{}", meta.language, params.service)
        } else {
            format!("/v2/{}/account", meta.language)
        };

        AnyResponder::Json(Json(LoginResponse {
            success: true,
            totp_required: None,
            redirect_url: Some(redirect_url),
            error: None,
        }))
    } else {
        AnyResponder::Json(Json(LoginResponse {
            success: false,
            totp_required: None,
            redirect_url: None,
            error: Some(strings.wrong_password.clone()),
        }))
    }
}