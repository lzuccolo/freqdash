// src/gui_adw/events/handlers.rs

use gtk4::{Box as GtkBox, TreeView};
use std::rc::Rc;
use std::cell::RefCell;

use crate::gui_adw::state::AppState;
use super::{query, filters, export}; // Importamos los módulos de eventos

// Renombramos la función para que sea más clara y aceptamos el TreeView
pub fn connect_all(
    left_panel: &GtkBox,
    right_panel: &GtkBox,
    tree_view: &TreeView,
    state: &Rc<RefCell<AppState>>,
) {
    // Pasamos el TreeView a los módulos que lo necesiten
    query::connect(left_panel, right_panel, tree_view, state);
    filters::connect(left_panel, state);
    export::connect(right_panel, tree_view, state);
}