use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use crate::core::locale::Messages;
use crate::core::object::{CodeType, LoginMeta};
use crate::core::state::AppState;
use crate::core::system_core::{AnyResponder};
use crate::global::{CORE};
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