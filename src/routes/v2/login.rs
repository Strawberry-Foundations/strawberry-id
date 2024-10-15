use std::fmt::Write;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::global::{ACCOUNTS, CONFIG, CORE, DATABASE};
use crate::core::locale::{Language, LANGUAGES, Messages};
use crate::core::object::{CodeType, LoginForm, LoginMeta};
use crate::core::params::{LoginParams, PostLoginParams};
use crate::core::state::AppState;
use crate::core::system_core::{Account, AnyResponder, verify_password};
use crate::core::uuid::generate_random_uuid;
use crate::utilities::name_parser;


pub async fn template_responder(
    strings: Messages, account: Account,
    params: PostLoginParams, uuid: String, external_params: String
) -> AnyResponder {

    AnyResponder::Template(Template::render("login", context! {
        title: &strings.login,
        strings: &strings,
        lang: &account.lang,
        meta: &account.meta,
        params: &params,
        redirect_after_login: &account.redirect_after_login,
        service_after_login: &account.service_after_login,
        uuid: &uuid,
        external_params: &external_params,
    }))
}

pub async fn catch_error(mut account: Account, error_message: &String, mut external_params: String) -> (Account, String){
    account.meta.error = true;
    account.meta.info_message = error_message.clone();
    account.meta.animation = String::from("none");

    if account.meta.redirect_after_login {
        write!(external_params, "?redirect={}", account.redirect_addr).unwrap()
    }
    else if account.meta.service_login {
        write!(external_params, "?service={}", account.params.service).unwrap();

        if account.params.oauth {
            write!(external_params, "&oauth=true").unwrap();
        }
    }

    (account, external_params)
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
pub async fn login(lang: &str, state: &State<AppState>, mut params: LoginParams) -> Template {
    let mut meta = LoginMeta {
        animation: String::from("slide-in-login"),
        ..Default::default()
    };

    meta.code = 0;

    if !LANGUAGES.contains(&lang) {
        return Template::render("404", context! {
            uri: format!("/v2/{lang}/login")
        });
    };

    meta.language = lang.to_string();

    let strings = state.messages.get(lang).cloned().unwrap();

    if !params.redirect.trim().is_empty() {
        meta.redirect_after_login = true;

        params.oauth = false;
        params.redirect = params.redirect.replace("http://", "").replace("https://", "");

        if params.hl.trim().is_empty() {
            params.hl = lang.to_string();
        }

        meta.trusted_web = CONFIG.vars.trusted_sites.contains(&params.redirect.to_lowercase());
    } else {
        meta.redirect_after_login = false;
    }

    if !params.service.trim().is_empty() && !meta.redirect_after_login {
        meta.service_login = !params.oauth;
        meta.service_name = name_parser(state, params.service.clone());
        meta.trusted_service = CONFIG.vars.trusted_services.contains(&params.service.to_lowercase());
    }
    else {
        meta.service_login = false;
    }

    if params.code != 0 || !params.code.to_string().len() < 10 {
        meta.code = params.code;
    }

    let redirect_after_login = strings.redirect_after_login.replace("%s", params.redirect.as_str());
    let service_after_login = strings.service_after_login.replace("%s", meta.service_name.as_str());

    let uuid = generate_random_uuid();

    ACCOUNTS.write().await.insert(
        uuid.clone(),
        Account {
            redirect_addr: params.redirect.clone(),
            lang: lang.to_string().clone(),
            meta: meta.clone(),
            params: params.clone(),
            redirect_after_login: redirect_after_login.clone(),
            service_after_login: service_after_login.clone()
        }
    );

    Template::render("login", context! {
        title: &strings.login,
        strings: &strings,
        lang: &lang,
        meta: &meta,
        params: &params,
        redirect_after_login: &redirect_after_login,
        service_after_login: &service_after_login,
        uuid: &uuid
    })
}




#[post("/login?<params..>", data = "<form>")]
pub async fn login_post(form: Form<LoginForm>, state: &State<AppState>, params: PostLoginParams, jar: &CookieJar<'_>) -> AnyResponder {
    let external_params = String::new();

    let username = &form.username;
    let password = &form.password;

    let binding = ACCOUNTS.read().await;
    let binding_account = binding.get(&params.uuid).unwrap();

    let account = Account::from_ref(binding_account);

    let uuid = generate_random_uuid();

    let strings = state.messages.get(account.lang.as_str()).cloned().unwrap();

    let user_base = match DATABASE.get_password(username.to_string()).await {
        Some(res) => res,
        None => {
            let (account, external_params) = catch_error(account, &strings.wrong_username, external_params).await;
            return template_responder(strings, account, params, uuid, external_params).await
        }
    };

    if verify_password(user_base.password, password) {
        let user_data = DATABASE.get_user_data(user_base.username).await;
        let data = user_data.clone();

        let code = CORE.write().await.generate_code(user_data, CodeType::Website);
        let uuid_code = CORE.write().await.generate_uuid_code(&code);

        if data.account_enabled == "false" {
            let (account, external_params) = catch_error(account, &strings.account_disabled, external_params).await;
            return template_responder(strings, account, params, uuid, external_params).await
        };

        jar.add_private(("_strawberryid.username", data.username.clone()));
        jar.add_private(("_strawberryid.email", data.email.clone()));
        jar.add_private(("_strawberryid.full_name", data.full_name.clone()));
        jar.add_private(("_strawberryid.profile_picture_url", data.profile_picture_url.clone()));

        if account.meta.code != 0 {
            return AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                    "/v2/{}/login/oauth_dialog/{}?code={}",
                    account.meta.language,
                    account.params.service,
                    account.meta.code
                ))
            ))
        }

        if account.meta.redirect_after_login {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                        "https://{}/callback?hl={}&code={}",
                        account.redirect_addr,
                        account.params.hl,
                        code
                    ))
            ))
        }
        else if account.meta.service_login {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                    "/v2/{}/login/service/{}?request_uuid={uuid_code}",
                    account.meta.language,
                    account.params.service,
                ))
            ))
        }
        else if account.params.oauth && !account.meta.service_login {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!(
                    "/v2/{}/login/oauth/{}",
                    account.meta.language,
                    account.params.service
                ))
            ))
        }
        else {
            AnyResponder::Redirect(Box::from(
                Redirect::to(format!("/v2/{}/account", account.meta.language))
            ))
        }
    }
    else {
        let (account, external_params) = catch_error(account, &strings.wrong_username_or_password, external_params).await;
        template_responder(strings, account, params, uuid, external_params).await
    }
}