use std::collections::HashMap;
use std::time::Duration;
use rocket::response::Redirect;
use rocket_dyn_templates::Template;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hex::encode;
use rand::Rng;
use sha2::{Sha256, Digest};

use crate::core::consts::PLACEHOLDER;
use crate::core::object::{CodeType, LoginMeta, UserData};
use crate::core::params::LoginParams;
use crate::utilities::unix_time;

#[derive(Debug, Responder)]
pub enum AnyResponder {
    Template(Template),
    Redirect(Box<Redirect>),
}

pub struct Account {
    pub redirect_addr: String,
    pub lang: String,
    pub meta: LoginMeta,
    pub params: LoginParams,
    pub redirect_after_login: String,
    pub service_after_login: String,
}

impl Account {
    pub fn from_ref(account: &Account) -> Self {
        Self {
            redirect_addr: account.redirect_addr.clone(),
            lang: account.lang.clone(),
            meta: account.meta.clone(),
            params: account.params.clone(),
            redirect_after_login: account.redirect_after_login.clone(),
            service_after_login: account.service_after_login.clone(),
        }
    }
}

pub type Codes = HashMap<u64, Vec<(UserData, u64, CodeType)>>;
pub type OAuthCodes = HashMap<u64, Vec<(String, UserData, u64, CodeType)>>;
pub type ServiceCodes = HashMap<u64, Vec<(UserData, u64, CodeType)>>;

#[derive(Default)]
pub struct Core {
    pub codes: Codes,
    pub oauth_codes: OAuthCodes,
    pub service_codes: ServiceCodes,
    pub uuid_codes: HashMap<String, u64>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
            oauth_codes: HashMap::new(),
            service_codes: HashMap::new(),
            uuid_codes: HashMap::new()
        }
    }

    pub fn generate_code(&mut self, user: UserData, code_type: CodeType) -> u64 {
        let mut code = rand::rng().random_range(1111111111..=9999999999);

        while self.codes.contains_key(&code) {
            code = self.generate_code(user.clone(), code_type)
        }

        let time = unix_time();
        let duration = Duration::from_secs(time) + Duration::from_secs(5 * 60);

        self.codes.insert(code, vec![(user.clone(), duration.as_secs(), code_type)]);

        code
    }

    pub fn generate_uuid_code(&mut self, code: &u64) -> String {
        let mut sha256 = Sha256::new();
        sha256.update(code.to_string().as_bytes());
        let result = sha256.finalize();

        let uuid = encode(result);

        self.uuid_codes.insert(uuid.clone(), *code);

        uuid
    }

    pub fn generate_code_ext(&mut self, code_type: CodeType) -> u64 {
        let mut code = rand::rng().random_range(1111111111..=9999999999);

        let user = UserData::default();

        match code_type {
            CodeType::Website => {
                return 0
            }
            CodeType::OAuth => {
                while self.oauth_codes.contains_key(&code) {
                    code = self.generate_code_ext(code_type)
                }

                let time = unix_time();
                let duration = Duration::from_secs(time) + Duration::from_secs(5 * 60);

                self.oauth_codes.insert(code, vec![(PLACEHOLDER.to_string(), user, duration.as_secs(), code_type)]);
            }
            CodeType::Service => {
                while self.service_codes.contains_key(&code) {
                    code = self.generate_code_ext(code_type)
                }

                let time = unix_time();
                let duration = Duration::from_secs(time) + Duration::from_secs(5 * 60);

                self.service_codes.insert(code, vec![(user, duration.as_secs(), code_type)]);
            }
        }

        code
    }
}

pub fn verify_password(stored_password: String, entered_password: &String) -> bool {
    let hash = PasswordHash::new(stored_password.as_str()).unwrap();

    let password: &[u8] = entered_password.as_bytes();

    Argon2::default().verify_password(password, &hash).is_ok()
}