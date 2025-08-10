// src/config.rs

use dotenvy::dotenv;
use std::env;
use crate::db::AsyncDatabase;


pub async fn init() {
    dotenv().ok();
    init_database().await;
}

pub fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL no está definida en .env")
}

pub async fn init_database() {
    let db_url = get_database_url();
    AsyncDatabase::init(&db_url).await.unwrap();
    println!("✅ AsyncDatabase inicializada correctamente.");
}

pub fn get_config_key(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} no está definida en .env"))
}
