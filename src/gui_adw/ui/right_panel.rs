use super::{status_bar, table_view, toolbar}; // Usar super:: para acceder a submódulos

use gtk4::prelude::*;
use gtk4::{Box, Orientation, PolicyType, ScrolledWindow, Separator, TreeModelFilter, TreeView};
use libadwaita::Bin;
use libadwaita::prelude::*;

/// Crea el panel derecho completo con tabla, toolbar y status bar
/// Retorna el panel, TreeView y TreeModelFilter para conectar eventos
pub fn create() -> (Box, TreeView, TreeModelFilter) {
    let panel = Box::new(Orientation::Vertical, 0);
    panel.set_hexpand(true);
    panel.add_css_class("view");

    // Usar Bin de Adwaita para mejor styling
    let content_bin = Bin::new();

    let content_box = Box::new(Orientation::Vertical, 0);

    // Toolbar con estilo Adwaita
    let toolbar = toolbar::create();
    content_box.append(&toolbar);
    content_box.append(&Separator::new(Orientation::Horizontal));

    // Crear store y filter model
    let store = table_view::create_list_store();
    let filter_model = TreeModelFilter::new(&store, None);
    filter_model.set_visible_column(30); // Columna de visibilidad

    // Crear tabla con estilo mejorado
    let table_view = table_view::create(&filter_model);

    // ScrolledWindow con estilo Adwaita
    let scrolled = ScrolledWindow::builder()
        .child(&table_view)
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .build();
    scrolled.add_css_class("card");
    content_box.append(&scrolled);

    // Status bar
    content_box.append(&Separator::new(Orientation::Horizontal));
    let status_bar = status_bar::create();
    content_box.append(&status_bar);

    content_bin.set_child(Some(&content_box));
    panel.append(&content_bin);

    (panel, table_view, filter_model)
}
