// src/config.rs

use once_cell::sync::OnceCell;
use std::env;

static DATABASE_URL: OnceCell<String> = OnceCell::new();

pub fn init_config() {
    dotenvy::dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL debe estar definida en el archivo .env");
    DATABASE_URL.set(url).expect("La configuración ya estaba inicializada");
}

pub fn get_database_url() -> &'static str {
    DATABASE_URL.get().expect("La configuración no está inicializada")
}