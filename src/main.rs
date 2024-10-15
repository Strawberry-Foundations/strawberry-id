#[macro_use] extern crate rocket;

use rocket::figment::Figment;
use rocket_dyn_templates::Template;

use crate::core::state::AppState;
use crate::global::CONFIG;

use crate::routes::v2::{
    index::{index, index_no_lang},
    login::{login, login_no_lang, login_post},
    logout::logout,
    login_oauth::{login_oauth, login_oauth_no_lang, login_oauth_post, login_oauth_lang_redir},
    oauth_dialog::{oauth_permit_dialog, oauth_permit_dialog_post},
    register::{register, register_no_lang},
    account::{account, account_no_lang},
};
use crate::routes::v2::api::{
    callback::{api_callback},
    oauth::callback::{api_oauth_callback},
    request::api_code_request,
    auth::api_auth,
};
use crate::routes::v1::index::{v1_index, v1_index_no_lang};
use crate::routes::root::{root, root_no_lang};
use crate::routes::web_static::static_files;

pub mod utilities;
pub mod global;
pub mod core;
pub mod routes;

#[rocket::main]
async fn main() {
    let config = CONFIG.to_rocket();

    let figment = Figment::from(config)
        .merge(("secret_key", CONFIG.vars.secret_key.as_str()))
        .merge(("template_dir", format!("src/{}", CONFIG.vars.template_folder)));

    let app_state = AppState::new().unwrap_or_else(|_| {
        panic!("Failed to load messages. Check the 'locales' folder.");
    });

    rocket::build()
        .manage(app_state)
        .mount("/v2", routes![
            index,
            index_no_lang,

            login,
            login_no_lang,
            login_post,

            login_oauth,
            login_oauth_no_lang,
            login_oauth_lang_redir,
            login_oauth_post,

            register,
            register_no_lang,

            oauth_permit_dialog,
            oauth_permit_dialog_post,
            
            account,
            account_no_lang,
        ])
        .mount("/v2/api", routes![
            api_callback,
            api_oauth_callback,

            api_code_request,
            api_auth
        ])
        .mount("/v1", routes![
            v1_index,
            v1_index_no_lang,
        ])
        .mount("/", routes![
            root,
            root_no_lang,
            static_files,
            logout,
        ])
        .configure(figment)
        .attach(Template::fairing())
        .launch()
        .await
        .unwrap();
}
