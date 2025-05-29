use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};

pub static LANGUAGES: [&str; 2] = ["de", "en"];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Messages {
    /// Basic strings
    pub login: String,
    pub login_to_sid: String,
    pub login_to_sid_subtitle: String,
    pub login_to: String,
    pub register: String,
    pub new_account: String,
    pub logout: String,
    pub close: String,
    pub password: String,
    pub email_username: String,
    pub username: String,
    pub email: String,
    pub email_optional: String,
    pub name_optional: String,
    pub code: String,
    pub str_continue: String,
    pub back: String,
    pub allow: String,
    pub goto_root: String,
    pub create_account_short: String,
    pub developer_documentation: String,
    pub my_profile: String,
    
    /// TOTP
    pub totp_required: String,
    pub totp_code: String,
    pub submit: String,
    pub wrong_totp_code: String,

    /// Content Strings
    pub strawberry_id_title: String,
    pub strawberry_id_subtitle: String,
    pub strawberry_id_subtitle_2: String,
    pub already_have_id: String,

    /// Benefits
    pub benefits_id: String,
    pub works_everywhere_title: String,
    pub works_everywhere_description: String,
    pub simple_use_title: String,
    pub simple_use_description: String,
    pub scaling_title: String,
    pub scaling_description: String,
    pub convinced: String,

    /// Footer
    pub privacy_notice: String,
    pub privacy_policy: String,
    pub terms_of_service: String,

    /// Account dropdown strings
    pub settings: String,
    pub my_products: String,
    pub cloud_engine: String,

    /// Login Strings
    pub create_account: String,
    pub create_account_2: String,

    /// Register Strings
    pub have_account: String,
    pub have_account_2: String,

    /// Login messages
    pub login_error: String,
    pub login_success: String,
    pub wrong_username_or_password: String,
    pub wrong_username: String,
    pub wrong_password: String,
    pub account_disabled: String,

    pub redirect_after_login: String,
    pub trusted_domain: String,
    pub not_trusted_domain: String,

    pub service_after_login: String,
    pub trusted_service: String,
    pub not_trusted_service: String,

    pub account_settings_after_login: String,

    pub continue_service_login: String,
    pub show_code: String,
    pub login_code: String,
    pub warning_code_show: String,

    /// Account settings
    pub account_settings: String,
    pub profile: String,
    pub security: String,
    pub account: String,
    pub notifications: String,
    pub member: String,

    pub security_settings: String,
    pub two_factor_auth: String,
    pub enable_2fa: String,
    pub disable_2fa: String,
    pub setup_totp: String,
    pub totp_setup_success: String,
    pub totp_enabled: String,
    pub totp_disabled: String,
    pub totp_setup_failed: String,
    pub totp_disable_failed: String,
    pub login_required: String,

    pub profile_settings: String,
    pub full_name: String,
    pub bio: String,

    /// General error messages
    pub cloud_engine_deactivated: String,

    /// OAuth Strings
    pub continue_code_input: String,
    pub error_no_code: String,
    pub error_invalid_code: String,
    pub code_valid: String,

    /// OAuth Dialog Strings
    pub service_wants_to_login: String,
    pub only_permit_you_know: String,
    pub login_not_possible: String,
    pub login_not_possible_invalid_code: String,
    pub login_success_subtitle: String,

    pub r#continue: String,
    pub welcome_back: String,
    pub user_not_found: String,
    pub connection_error: String,
}

#[derive(Debug)]
pub enum Language {
    German,
    English,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Language {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let header: Option<&str> = request.headers().get_one("Accept-Language");
        if let Some(header) = header {
            let lang_str = header;
            if lang_str.starts_with("de") {
                Outcome::Success(Language::German)
            } else if lang_str.starts_with("en") {
                Outcome::Success(Language::English)
            } else {
                Outcome::Forward(rocket::http::Status::Continue) // Unsupported language
            }
        } else {
            Outcome::Forward(rocket::http::Status::Continue) // Language header not present
        }
    }
}