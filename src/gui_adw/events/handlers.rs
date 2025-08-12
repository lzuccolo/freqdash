// src/gui_adw/events/handlers.rs

use super::{connect_export, connect_filters, connect_query}; // Importar desde el módulo padre
use crate::gui_adw::state::AppState;
use gtk4::{Box, TreeView};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Función principal para conectar todos los eventos
pub fn connect_all(
    left_panel: &Box,
    right_panel: &Box,
    table_view: &TreeView,
    state: &Rc<RefCell<AppState>>//,
    //rt: Arc<Runtime>,
) {
    //connect_query(left_panel, right_panel, state, rt.clone());
    connect_query(left_panel, right_panel, state);
    connect_filters(left_panel, state);
    connect_export(right_panel, table_view, state);
}
