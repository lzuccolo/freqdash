// src/gui_adw/events/mod.rs

pub mod query;
pub mod filters;
pub mod export;
pub mod handlers;

// Re-exportar funciones públicas
pub use handlers::connect_all;
pub use query::connect as connect_query;
pub use filters::connect as connect_filters;
pub use export::connect as connect_export;
