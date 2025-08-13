// src/gui_adw/events/mod.rs

pub mod export;
pub mod filters;
pub mod handlers;
pub mod query;

// Re-exportar funciones públicas
// pub use export::connect as connect_export;
// pub use filters::connect as connect_filters;
pub use handlers::connect_all;
//pub use query::connect as connect_query;
