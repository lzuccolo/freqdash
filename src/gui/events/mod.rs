mod query;
mod filters;
mod export;

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::{Box, TreeView};
use crate::gui::state::AppState;

/// Conecta todos los eventos de la aplicación
pub fn connect_all(
    left_panel: &Box,
    right_panel: &Box,
    table_view: &TreeView,
    state: &Rc<RefCell<AppState>>
) {
    // Conectar eventos de consulta
    query::connect(left_panel, right_panel, state);
    
    // Conectar eventos de filtros
    filters::connect(left_panel, state);
    
    // Conectar eventos de exportación
    export::connect(right_panel, table_view, state);
}