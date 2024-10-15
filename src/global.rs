use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use tokio::sync::RwLock;

use lazy_static::lazy_static;
use sqlx::{Pool, Sqlite};

use crate::core::config::RouterConfig;
use crate::core::db::Database;
use crate::core::system_core::{Account, Core};

lazy_static! {
    pub static ref CONFIG: RouterConfig = {
        let exe_path = env::current_exe().unwrap_or_else(|_| {
            eprintln!("Could not get your Strawberry ID Runtime Executable");
            std::process::exit(1);
        });

        let exe_dir = exe_path.parent().unwrap_or_else(|| {
            eprintln!("Could not get directory of your Strawberry ID Runtime Executable");
            std::process::exit(1);
        });

        let exe_dir_str = PathBuf::from(exe_dir).display().to_string();

        let mut config_path = format!("{exe_dir_str}/config.yml");

        if !Path::new(&config_path).exists() {
            config_path = String::from("./config.yml");
        }

        RouterConfig::new(config_path)
    };

    pub static ref ACCOUNTS: RwLock<HashMap<String, Account>> = RwLock::new(HashMap::new());
    pub static ref CORE: RwLock<Core> = RwLock::new(Core::new());

    pub static ref DATABASE: Database = futures::executor::block_on(async {
        Database::new("./data/data.db").await
    });

    pub static ref CONNECTION: Pool<Sqlite> = DATABASE.connection.clone();
}