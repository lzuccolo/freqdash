// src/db.rs

use once_cell::sync::OnceCell;
use deadpool_postgres::{Pool, Config, Runtime};
use tokio_postgres::NoTls;
use crate::config;

static DB_POOL: OnceCell<Pool> = OnceCell::new();

pub fn init_db_pool() {
    let mut cfg = Config::new();
    cfg.url = Some(config::get_database_url().to_string());
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("No se pudo crear el pool de la base de datos");
    
    DB_POOL.set(pool).expect("El pool de la base de datos ya estaba inicializado");
    println!("✅ Pool de conexiones a la DB inicializado correctamente.");
}

pub fn get_db_pool() -> &'static Pool {
    DB_POOL.get().expect("El pool de la base de datos no está inicializado")
}