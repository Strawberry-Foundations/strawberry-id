use rocket::response::content::RawJson;
use serde_json::json;
use sqlx::Row;

use crate::core::consts::{INVALID_TOKEN, INVALID_USERNAME, NO_CREDENTIALS};
use crate::core::object::UserData;
use crate::core::params::AuthParams;
use crate::utilities::generate_hash;
use crate::global::{DATABASE};

#[get("/auth?<params..>")]
pub async fn api_auth(params: AuthParams) -> RawJson<String> {
    let username = params.username;
    let token = params.token;

    if username.trim().is_empty() || token.trim().is_empty() {
        return RawJson(String::from(NO_CREDENTIALS))
    }

    let user_data = sqlx::query("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_all(&DATABASE.connection)
        .await.unwrap();

    let user_data = match user_data.first() {
        Some(res) => res,
        None => return RawJson(String::from(INVALID_USERNAME))
    };

    let user = UserData {
        username: user_data.get("username"),
        password: user_data.get("password"),
        email: user_data.get("email"),
        full_name: user_data.get("full_name"),
        profile_picture_url: user_data.get("profile_picture_url"),
        account_enabled: user_data.get("account_enabled"),
        cloud_engine_enabled: user_data.get("cloud_engine_enabled"),
        strawberry_one: user_data.get("strawberry_one"),
        totp_enabled: user_data.get("totp_enabled"),
        totp_secret: user_data.get("totp_secret"),
    };

    let stored_token = generate_hash(user.password);

    if token != stored_token {
        return RawJson(String::from(INVALID_TOKEN))
    }


    let json = json!({
        "data": {
            "status": "Ok",
            "user": {
                "username": user.username,
                "full_name": user.full_name,
                "email": user.email,
                "profile_picture_url": user.profile_picture_url,
                "account_enabled": user.account_enabled
            }
        }
    });


    let json_string = serde_json::to_string_pretty(&json).unwrap();

    RawJson(json_string)
}
