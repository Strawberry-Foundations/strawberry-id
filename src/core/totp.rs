use totp_rs::{Algorithm, Secret, TOTP};

pub struct StrawberryIdTotp {
    pub secret_base32: String,
    pub secret: String,
    pub qr_code: String,
}

impl StrawberryIdTotp {
    pub fn setup(account_name: &str) -> Self {
        let base_token = Secret::generate_secret().to_string();
        let secret = Secret::Raw(base_token.as_bytes().to_vec()).to_bytes().unwrap();

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            Some("Strawberry ID".to_string()),
            account_name.to_string()
        );

        let totp = totp.unwrap();
        let qr_code = totp.get_qr_base64().unwrap();
        let secret_base32 = totp.get_secret_base32();

        Self {
            secret_base32,
            secret: base_token.to_string(),
            qr_code
        }
    }

    pub fn check(secret: &str, code: &str) -> bool {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Raw(secret.as_bytes().to_vec()).to_bytes().unwrap(),
            Some("Strawberry ID".to_string()),
            "".to_string()
        );

        let totp = totp.unwrap();
        let token = totp.generate_current().unwrap();
        
        token == *code
    }
}