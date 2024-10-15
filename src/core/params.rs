#![allow(clippy::blocks_in_conditions, clippy::useless_conversion)]
use rocket::serde::{Deserialize, Serialize};

#[derive(FromForm, Deserialize, Serialize, Clone)]
pub struct LoginParams {
    #[field(name = "redirect", default = "")]
    pub redirect: String,

    #[field(name = "hl", default = "")]
    pub hl: String,

    #[field(name = "service", default = "")]
    pub service: String,

    #[field(name = "oauth", default = false)]
    pub oauth: bool,

    #[field(name = "debug", default = false)]
    pub debug: bool,

    #[field(name = "code", default = 0)]
    pub code: u64
}

#[derive(FromForm, Deserialize, Serialize, Clone)]
pub struct LogoutParams {
    #[field(name = "hl", default = "")]
    pub hl: String,
}

#[derive(FromForm, Deserialize, Serialize)]
pub struct PostLoginParams {
    #[field(name = "uuid")]
    pub uuid: String,
}

#[derive(FromForm, Deserialize, Serialize)]
pub struct ApiParams {
    #[field(name = "code")]
    pub code: u64,
}

#[derive(FromForm, Deserialize, Serialize)]
pub struct AuthParams {
    #[field(name = "username", default = "")]
    pub username: String,

    #[field(name = "token", default = "")]
    pub token: String,
}