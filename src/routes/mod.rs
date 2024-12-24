pub mod root;
pub mod web_static;
pub mod status_code;

pub mod v2 {
    pub mod index;
    pub mod login;
    pub mod login_oauth;
    pub mod register;
    pub mod logout;
    pub mod oauth_dialog;
    pub mod account;
    pub mod totp;

    pub mod api {
        pub mod callback;
        pub mod request;
        pub mod auth;

        pub mod oauth {
            pub mod callback;
        }
    }
}

pub mod v1 {
    pub mod index;
}
