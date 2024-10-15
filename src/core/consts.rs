pub const CODE_EXPIRED: &str = r#"{"data": { "status": "Code expired" }}"#;
pub const INVALID_CODE: &str = r#"{"data": { "status": "Invalid code" }}"#;
pub const NOT_AUTHENTICATED: &str = r#"{"data": { "status": "Not authenticated" }}"#;
pub const WRONG_CODE_TYPE_OAUTH: &str = r#"{"data": { "status": "Code type is not for OAuth" }}"#;
pub const WRONG_CODE_TYPE_WEB: &str = r#"{"data": { "status": "Code type is not for Web" }}"#;
pub const NO_CREDENTIALS: &str = r#"{"data": { "status": "No credentials given" }}"#;
pub const INVALID_USERNAME: &str = r#"{"data": { "status": "Invalid username" }}"#;
pub const INVALID_TOKEN: &str = r#"{"data": { "status": "Invalid token" }}"#;

pub const PLACEHOLDER: &str =".strawberry_id_admin";