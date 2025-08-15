// src/gui/ui/right_panel.rs

use super::{status_bar, table_view};
use gtk4::prelude::*;
use gtk4::{Box, Orientation, PolicyType, ScrolledWindow, Separator, TreeModelFilter, TreeView};

pub fn create() -> (Box, TreeView, TreeModelFilter) {
    let content_box = Box::new(Orientation::Vertical, 0);
    content_box.set_hexpand(true);
    content_box.add_css_class("view");

    let store = table_view::create_list_store();
    let filter_model = TreeModelFilter::new(&store, None);
    filter_model.set_visible_column(30);

    let table_view = table_view::create(&filter_model);

    let scrolled = ScrolledWindow::builder()
        .child(&table_view)
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .build();
    scrolled.add_css_class("card");
    content_box.append(&scrolled);

    content_box.append(&Separator::new(Orientation::Horizontal));

    // Usamos el módulo status_bar para crear la barra de estado.
    let status_bar = status_bar::create();
    content_box.append(&status_bar);

    (content_box, table_view, filter_model)
}
