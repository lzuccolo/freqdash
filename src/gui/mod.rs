// src/gui/mod.rs

mod app;
mod state;
mod ui;
mod events;
pub mod utils; // Hacerlo público para que sea accesible

use crate::config;

/// Punto de entrada principal para la aplicación GUI
pub fn run() {
    // Inicializar runtime de Tokio para operaciones async
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");
    
    // Inicializar configuración y base de datos
    rt.block_on(config::init());
    
    // Lanzar la aplicación GTK
    app::launch();
}