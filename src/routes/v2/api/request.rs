use rocket::response::content::RawHtml;

use crate::core::object::CodeType;
use crate::global::CORE;

#[get("/request")]
pub async fn api_code_request() -> RawHtml<String> {
    let code = CORE.write().await.generate_code_ext(CodeType::OAuth);

    RawHtml(code.to_string())
}
