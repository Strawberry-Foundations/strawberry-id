use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use crate::core::locale::Messages;
use crate::core::object::{CodeType, LoginMeta, UserData};
use crate::core::state::AppState;
use crate::core::system_core::{verify_password, AnyResponder};
use crate::global::{CORE, DATABASE};
use crate::core::params::LoginParams;
use crate::core::totp::StrawberryIdTotp;
use crate::routes::v2::login::{template_responder, TEMP_STORAGE};

#[derive(FromForm)]
pub struct TotpForm {
    pub totp_code: String,
}

#[post("/<lang>/login/totp?<params..>", data = "<form>")]
pub async fn login_totp(
    lang: &str, form: Form<TotpForm>, state: &State<AppState>,
    mut params: LoginParams, jar: &CookieJar<'_>, uri: &rocket::http::uri::Origin<'_>
) -> AnyResponder {
    let meta = LoginMeta::collect(&mut params, lang, state);

    let strings = state.messages.get(lang).cloned().unwrap();
    let full_params = match uri.query() {
        Some(query) => query.to_string(),
        None => String::new()
    };

    let temp_token = jar.get_private("_strawberryid.temp_token").map(|c| c.value().to_string());

    if let Some(temp_token) = temp_token {
        let user_data = TEMP_STORAGE.read().await.get(&temp_token).cloned();
        if let Some(user_data) = user_data {
            return if StrawberryIdTotp::check(&user_data.totp_secret.clone(), &form.totp_code) {
                let code = CORE.write().await.generate_code(user_data.clone(), CodeType::Website);

                jar.add_private(rocket::http::Cookie::new("_strawberryid.username", user_data.username.clone()));
                jar.add_private(rocket::http::Cookie::new("_strawberryid.email", user_data.email.clone()));
                jar.add_private(rocket::http::Cookie::new("_strawberryid.full_name", user_data.full_name.clone()));
                jar.add_private(rocket::http::Cookie::new("_strawberryid.profile_picture_url", user_data.profile_picture_url.clone()));

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
                            lang,
                            params.service
                        ))
                    ))
                }
                else {
                    AnyResponder::Redirect(Box::from(
                        Redirect::to(format!("/v2/{}/account", lang))
                    ))
                }
            } else {
                let mut meta = LoginMeta::collect(&mut params, lang, state);
                meta.info_message = strings.wrong_totp_code.clone();
                meta.error = true;
                template_responder(&strings, lang, &params, &meta, &full_params, &params.redirect, &params.service).await
            }
        }
    }
    AnyResponder::Redirect(Box::from(Redirect::to(format!("/v2/{}/login", lang))))
}


#[derive(FromForm)]
pub struct DisableTotpForm {
    pub password: String,
    pub totp_code: String,
}

#[post("/<lang>/account/totp/disable", data = "<form>")]
pub async fn disable_totp(
    lang: &str, form: Form<DisableTotpForm>, state: &State<AppState>, jar: &CookieJar<'_>
) -> AnyResponder {
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

    if !is_authenticated {
        return AnyResponder::Template(Template::render("login", context! {
            title: &strings.login,
            strings: &strings,
            lang: &lang,
            error: &strings.login_required,
        }));
    }

    if !StrawberryIdTotp::check(&user.totp_secret.clone(), &form.totp_code) {
        return AnyResponder::Template(Template::render("account", context! {
            title: &strings.account_settings,
            strings: &strings,
            lang: &lang,
            is_authenticated: is_authenticated,
            user: user,
            action: "disable_totp_failed",
        }));
    }

    if !verify_password(user.password.clone(), &form.password) {
        return AnyResponder::Template(Template::render("account", context! {
            title: &strings.account_settings,
            strings: &strings,
            lang: &lang,
            is_authenticated: is_authenticated,
            user: user,
            action: "disable_totp_failed",
        }));
    }

    DATABASE.set_totp(user.username.clone(), "false".to_string()).await.unwrap();

    AnyResponder::Template(Template::render("account", context! {
        title: &strings.account_settings,
        strings: &strings,
        lang: &lang,
        is_authenticated: is_authenticated,
        user: user,
        action: "disable_totp_success",
    }))
}