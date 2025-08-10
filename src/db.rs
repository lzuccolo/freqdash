use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls};

pub struct AsyncDatabase;

// Global singleton con Mutex asincrónico y Arc para clonarlo
static INSTANCE: OnceCell<Arc<Mutex<Client>>> = OnceCell::new();

impl AsyncDatabase {
    pub async fn init(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;

        // Lanzamos la conexión en segundo plano
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Error conexión DB: {}", e);
            }
        });

        // Guardamos el cliente envuelto en Arc<Mutex>
        INSTANCE.set(Arc::new(Mutex::new(client))).map_err(|_| "Ya inicializado")?;
        println!("✅ AsyncDatabase inicializada correctamente.");
        Ok(())
    }

    // Devolvemos una copia del Arc para que pueda ser utilizado de forma segura
    pub async fn get_client() -> Arc<Mutex<Client>> {
        INSTANCE.get().expect("Base de datos no inicializada").clone()
    }
}
