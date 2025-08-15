// src/gui/init.rs

use crate::config;
use crate::gui::app;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Punto de entrada principal para la aplicación GUI con libadwaita
pub fn run() {
    // Crear runtime de Tokio compartido con configuración optimizada
    let rt = Arc::new(
        Runtime::new().expect("Failed to create Tokio runtime")
    );
    
    // Inicializar configuración y base de datos
    rt.block_on(config::init());
    
    // Lanzar la aplicación Adwaita con el runtime
    app::launch(rt);
}