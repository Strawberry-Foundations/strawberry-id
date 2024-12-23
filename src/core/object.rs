#![allow(clippy::blocks_in_conditions, clippy::useless_conversion)]

use rocket::State;
use serde::{Deserialize, Serialize};
use crate::core::params::LoginParams;
use crate::core::state::AppState;
use crate::global::CONFIG;
use crate::utilities::name_parser;

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct LoginMeta {
    /// Error & information
    pub error: bool,
    pub info_message: String,

    /// Flags
    pub redirect_after_login: bool,
    pub service_after_login: bool,
    pub trusted_web: bool,
    pub trusted_service: bool,

    /// Generic strings
    pub service_name: String,
    pub language: String,

    /// Vars
    pub code: u64,
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct OAuthMeta {
    /// Error & information
    pub error: bool,
    pub info_message: String,

    /// Flags
    pub trusted: bool,

    /// Generic strings
    pub service_name: String,
    pub service: String,
    pub animation: String,

}

#[derive(Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodeType {
    #[default]
    Website,
    OAuth,
    Service,
}

#[derive(FromForm)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub username: String,
    pub password: String,
    pub email: String,
    pub full_name: String,
    pub profile_picture_url: String,
    pub account_enabled: String,
    pub cloud_engine_enabled: String,
    pub strawberry_one: String
}

#[derive(FromForm)]
pub struct OAuthForm {
    #[field(name = "code", default = 0)]
    pub code: u64,
}

impl LoginMeta {
    pub fn collect(params: &mut LoginParams, lang: &str, state: &State<AppState>) -> Self {
        let mut meta = LoginMeta::default();
        meta.code = 0;
        meta.language = lang.to_string();

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
            meta.service_after_login = !params.oauth;
            meta.service_name = name_parser(state, params.service.clone());
            meta.trusted_service = CONFIG.vars.trusted_services.contains(&params.service.to_lowercase());
        }
        else {
            meta.service_after_login = false;
        }

        if params.code != 0 || !params.code.to_string().len() < 10 {
            meta.code = params.code;
        }
        
        meta
    }
}