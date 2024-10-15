#![allow(clippy::blocks_in_conditions, clippy::useless_conversion)]
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct LoginMeta {
    /// Error & information
    pub error: bool,
    pub info_message: String,

    /// Flags
    pub redirect_after_login: bool,
    pub service_login: bool,
    pub trusted_web: bool,
    pub trusted_service: bool,

    /// Generic strings
    pub service_name: String,
    pub animation: String,
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
}

#[derive(FromForm)]
pub struct OAuthForm {
    #[field(name = "code", default = 0)]
    pub code: u64,
}