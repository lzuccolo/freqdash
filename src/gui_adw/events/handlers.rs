// src/gui_adw/events/handlers.rs
use libadwaita::HeaderBar;
use gtk4::{Box as GtkBox, TreeView};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc; // <-- Use the standard library's MPSC

use super::{export, filters, query};
use crate::gui_adw::app::DatabaseCommand;
use crate::gui_adw::state::AppState;

pub fn connect_all(
    left_panel: &GtkBox,
    right_panel: &GtkBox,
    header_bar: &HeaderBar,
    tree_view: &TreeView,
    state: &Rc<RefCell<AppState>>,
    command_tx: mpsc::Sender<DatabaseCommand>, // <-- Type is now correct
) {
    query::connect(
        left_panel,
        right_panel,
        header_bar,
        tree_view,
        state,
        command_tx.clone(),
    );
    filters::connect(left_panel, state);
    export::connect(header_bar, right_panel, tree_view, state);
}
