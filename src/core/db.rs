use sqlx::{Pool, Row, Sqlite};
use sqlx::sqlite::SqlitePool;
use crate::core::object::UserData;

pub struct Database {
    pub connection: Pool<Sqlite>,
    pub location: String,
}

pub struct UBaseStruct {
    pub username: String,
    pub password: String,
}

impl Database {
    pub async fn new(path: impl ToString) -> Self {
        let sqlite_db = SqlitePool::connect(&path.to_string()).await.unwrap_or_else(|_| {
            eprintln!("Could not open database");
            std::process::exit(1);
        });

        Self {
            connection: sqlite_db,
            location: path.to_string()
        }
    }

    pub async fn get_password(&self, username: String) -> Option<UBaseStruct> {
        match sqlx::query("SELECT username, password FROM users WHERE username = ?")
            .bind(username.clone())
            .fetch_one(&self.connection)
            .await {
            Ok(row) => {
                Some(UBaseStruct {
                    username: row.get("username"),
                    password: row.get("password")
                })
            },
            Err(_) => {
                match sqlx::query("SELECT username, password FROM users WHERE email = ?")
                    .bind(username)
                    .fetch_one(&self.connection)
                    .await {
                    Ok(row) => {
                        Some(UBaseStruct {
                            username: row.get("username"),
                            password: row.get("password")
                        })
                    }
                    Err(_) => None,
                }
            }
        }
    }

    pub async fn get_user_data(&self, username: String) -> UserData {
        let user_data = sqlx::query("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_all(&self.connection)
            .await.unwrap();

        let user_data = user_data.first().unwrap();

        UserData {
            username: user_data.get("username"),
            password: user_data.get("password"),
            email: user_data.get("email"),
            full_name: user_data.get("full_name"),
            profile_picture_url: user_data.get("profile_picture_url"),
            account_enabled: user_data.get("account_enabled"),
            cloud_engine_enabled: user_data.get("cloud_engine_enabled"),
        }
    }
}