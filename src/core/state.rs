use std::collections::HashMap;
use std::path::Path;
use serde::de::Error;

use crate::core::locale::{Messages, LANGUAGES};

pub struct AppState {
    pub messages: HashMap<String, Messages>,
    pub services: HashMap<String, String>,
}

impl AppState {
    pub fn new() -> Result<AppState, serde_yaml::Error> {
        let messages = Self::get_messages()?;
        let services = Self::get_services().unwrap();

        Ok(AppState {
            messages,
            services
        })
    }

    pub fn get_messages() -> Result<HashMap<String, Messages>, serde_yaml::Error> {
        let mut messages_map = HashMap::new();

        for lang in LANGUAGES {
            let lang_path_string = match lang {
                "de" => "de_DE",
                "en" => "en_US",
                _ => ""
            };

            let path = format!("src/locales/{lang_path_string}.yml");

            if !Path::new(&path).exists() {
                return Err(serde_yaml::Error::custom(format!("Language file not found: {}", lang)));
            }

            let yaml_str = std::fs::read_to_string(&path).unwrap();

            let messages: Messages = serde_yaml::from_str(&yaml_str)?;

            messages_map.insert(lang.to_string(), messages);
        }

        Ok(messages_map)
    }

    pub fn get_services() -> Result<HashMap<String, String>, serde_json::Error> {
        let mut services_map = HashMap::new();

        let path = "data/services.json".to_string();

        if !Path::new(&path).exists() {
            return Err(serde_json::Error::custom("services.json not found"));
        }

        let services_str = std::fs::read_to_string(&path).unwrap();
        let mut services: serde_json::Value = serde_json::from_str(&services_str)?;

        for (service, name) in services.as_object_mut().unwrap() {
            services_map.insert(service.to_string(), name.as_str().unwrap().to_string());
        }

        Ok(services_map)
    }
}