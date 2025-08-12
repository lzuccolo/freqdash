// src/gui_adw/events/mod.rs

// Submódulos privados
mod query;
mod filters;
mod export;
mod handlers;

// Re-exportar funciones públicas
pub use handlers::connect_all;
pub use query::connect as connect_query;
pub use filters::connect as connect_filters;
pub use export::connect as connect_export;
