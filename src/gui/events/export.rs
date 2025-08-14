// src/gui_adw/events/export.rs

use adw::HeaderBar;
use gtk4::prelude::*;
use gtk4::{Button, TreeModelFilter, TreeView}; // <-- FIX: Added TreeModelFilter
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backtest::logic::export_summary_to_csv;
use crate::backtest::model::StrategyGridRow;
use crate::gui_adw::state::AppState;
use crate::gui_adw::utils;

pub fn connect(
    header_bar: &HeaderBar,
    right_panel: &gtk4::Box,
    table_view: &TreeView,
    state: &Rc<RefCell<AppState>>,
) {
    let export_selection: Button = utils::find_widget(header_bar, "export_selection");
    let export_all: Button = utils::find_widget(header_bar, "export_all");
    let clear: Button = utils::find_widget(header_bar, "clear");

    connect_export_selection(&export_selection, table_view, state);
    connect_export_all(&export_all, state);
    connect_clear(&clear, header_bar, table_view, state);
}

fn connect_export_selection(button: &Button, table_view: &TreeView, state: &Rc<RefCell<AppState>>) {
    let state_clone = state.clone();
    let selection = table_view.selection();

    button.connect_clicked(move |_| {
        let (selected_rows, model) = selection.selected_rows();
        if selected_rows.is_empty() {
            println!("No rows selected for export.");
            return;
        }

        let state = state_clone.borrow();
        let selected_indices: Vec<usize> = selected_rows
            .iter()
            .filter_map(|path| path.indices().get(0).copied().map(|i| i as usize))
            .collect();

        let mut selected_data = Vec::new();
        for index in selected_indices {
            if let Some(iter) = model.iter_nth_child(None, index as i32) {
                if let Ok(strategy_name) = model.get_value(&iter, 0).get::<String>() {
                    if let Some(row_data) =
                        state.results.iter().find(|r| r.strategy == strategy_name)
                    {
                        selected_data.push(row_data.clone());
                    }
                }
            }
        }

        if !selected_data.is_empty() {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("selection_{}.csv", timestamp);
            if let Err(e) = export_summary_to_csv(&selected_data, &filename) {
                eprintln!("Error exporting selection to CSV: {}", e);
            } else {
                println!(
                    "✅ Exportado {} estrategias a {}",
                    selected_data.len(),
                    filename
                );
            }
        }
    });
}

fn connect_export_all(button: &Button, state: &Rc<RefCell<AppState>>) {
    let state_clone = state.clone();
    button.connect_clicked(move |_| {
        let state = state_clone.borrow();
        if state.results.is_empty() {
            println!("No results to export.");
            return;
        }
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("all_{}.csv", timestamp);
        if let Err(e) = export_summary_to_csv(&state.results, &filename) {
            eprintln!("Error exporting all to CSV: {}", e);
        } else {
            println!(
                "✅ Exportado {} estrategias a {}",
                state.results.len(),
                filename
            );
        }
    });
}

fn connect_clear(
    button: &Button,
    header_bar: &HeaderBar,
    tree_view: &TreeView,
    state: &Rc<RefCell<AppState>>,
) {
    let state_clone = state.clone();
    let header_bar_clone = header_bar.clone();
    let tree_view_clone = tree_view.clone();

    button.connect_clicked(move |_| {
        println!("Limpiando resultados...");
        let mut state = state_clone.borrow_mut();

        let column_types: Vec<glib::Type> = (0..state.store.n_columns())
            .map(|i| state.store.column_type(i))
            .collect();

        let new_store = gtk4::ListStore::new(&column_types);
        let new_filter_model = TreeModelFilter::new(&new_store, None);
        new_filter_model.set_visible_column(30);

        tree_view_clone.set_model(Some(&new_filter_model));

        state.store = new_store;
        state.filter_model = new_filter_model;
        state.results.clear();

        let results_label: gtk4::Label = utils::find_widget(&header_bar_clone, "results_count");
        results_label.set_markup("<b>Resultados:</b> 0");

        let export_selection: Button = utils::find_widget(&header_bar_clone, "export_selection");
        let export_all: Button = utils::find_widget(&header_bar_clone, "export_all");
        export_selection.set_sensitive(false);
        export_all.set_sensitive(false);

        println!("Limpieza completada.");
    });
}
