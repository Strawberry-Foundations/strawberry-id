use rocket::response::content::RawJson;
use serde_json::json;

use crate::core::consts::{CODE_EXPIRED, INVALID_CODE, WRONG_CODE_TYPE_WEB};
use crate::core::object::CodeType;
use crate::core::params::ApiParams;
use crate::utilities::unix_time;
use crate::global::CORE;

#[get("/callback?<params..>")]
pub async fn api_callback(params: ApiParams) -> RawJson<String> {
    let code = params.code;

    let binder = CORE.read().await;

    let (user, expiration_time, code_type) = match binder.codes.get(&code) {
        Some(time) => {
            let binder = time.first().unwrap();

            (&binder.0, binder.1, binder.2)
        },
        None => return RawJson(String::from(INVALID_CODE))
    };

    let user = user.clone();

    drop(binder);


    let current_time = unix_time();
    let expired = current_time > expiration_time;

    if expired {
        CORE.write().await.codes.remove(&code).unwrap();
        return RawJson(String::from(CODE_EXPIRED))
    }

    if code_type != CodeType::Website {
        return RawJson(String::from(WRONG_CODE_TYPE_WEB))
    }

    let json = json!({
        "data": {
            "status": "Ok",
            "code_meta": {
                "expiration_time": expiration_time,
                "current_time": current_time,
                "type": code_type,
            },
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

    CORE.write().await.codes.remove(&code).unwrap();

    RawJson(json_string)
}
