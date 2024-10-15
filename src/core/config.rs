use serde::Deserialize;
use serde_yaml::from_str;

use crate::utilities;

#[derive(Debug, Deserialize)]
pub struct HostRouterConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct VarsRouterConfig {
    pub static_url_path: String,
    pub static_folder: String,
    pub template_folder: String,
    pub trusted_sites: Vec<String>,
    pub trusted_services: Vec<String>,
    pub secret_key: String,
}

#[derive(Deserialize)]
pub struct RouterConfig {
    pub host: HostRouterConfig,
    pub vars: VarsRouterConfig,

    #[serde(skip)]
    pub path: String,
}

impl RouterConfig {
    pub fn new(config_path: String) -> Self {
        let cfg_content = utilities::open_config(&config_path);

        let mut config: Self = from_str(&cfg_content).unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(1);
        });

        config.path = config_path;

        config
    }

    pub fn to_rocket(&self) -> rocket::Config {
        rocket::Config {
            address: self.host.address.clone().parse().unwrap(),
            port: self.host.port,
            ..Default::default()
        }
    }
}