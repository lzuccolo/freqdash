// src/gui_adw/ui/mod.rs

pub mod left_panel;
pub mod right_panel;
pub mod table_view;
pub mod toolbar;
pub mod status_bar;

// Re-exportar funciones públicas
pub use left_panel::create as create_left_panel;
pub use right_panel::create as create_right_panel;
pub use table_view::{create_list_store, get_base_store};