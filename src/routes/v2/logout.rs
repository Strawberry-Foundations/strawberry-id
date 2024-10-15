use rocket::http::CookieJar;
use rocket::response::Redirect;

use crate::core::params::LogoutParams;

#[get("/logout?<params..>")]
pub async fn logout(params: LogoutParams, jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("_strawberryid.username");
    jar.remove_private("_strawberryid.email");
    jar.remove_private("_strawberryid.full_name");
    jar.remove_private("_strawberryid.profile_picture_url");

    Redirect::to(format!("/v2/{}", params.hl))
}