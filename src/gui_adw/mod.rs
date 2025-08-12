// src/gui_adw/mod.rs

mod app;
mod state;
mod ui;
mod events;
mod init;
pub mod utils;

// Re-exportar la función run desde init
pub use init::run;