pub mod left_panel;
pub mod table_view;
pub mod toolbar;
pub mod status_bar;

use gtk4::prelude::*;
use gtk4::{Box, Orientation, PolicyType, ScrolledWindow, Separator, TreeModelFilter, TreeView};

/// Crea el panel derecho completo con tabla, toolbar y status bar
/// Retorna el panel, TreeView y TreeModelFilter para conectar eventos
pub fn create_right_panel() -> (Box, TreeView, TreeModelFilter) {
    let panel = Box::new(Orientation::Vertical, 0);
    panel.set_hexpand(true);
    
    // Toolbar
    let toolbar = toolbar::create();
    panel.append(&toolbar);
    panel.append(&Separator::new(Orientation::Horizontal));
    
    // Crear store y filter model
    let store = table_view::create_list_store();
    let filter_model = TreeModelFilter::new(&store, None);
    filter_model.set_visible_column(30); // Columna de visibilidad
    
    // Crear tabla
    let table_view = table_view::create(&filter_model);
    
    // Tabla con scroll
    let scrolled = ScrolledWindow::builder()
        .child(&table_view)
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .build();
    panel.append(&scrolled);
    
    // Status bar
    panel.append(&Separator::new(Orientation::Horizontal));
    let status_bar = status_bar::create();
    panel.append(&status_bar);
    
    (panel, table_view, filter_model)
}