// src/gui/events/export.rs

use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use chrono::Local;

use crate::gui::state::AppState;
use crate::gui::utils;
use crate::backtest::logic::export_summary_to_csv;

/// Conecta los eventos de exportación
pub fn connect(
    right_panel: &gtk4::Box,
    table_view: &gtk4::TreeView,
    state: &Rc<RefCell<AppState>>
) {
    let export_selection: gtk4::Button = utils::find_widget(right_panel, "export_selection");
    let export_all: gtk4::Button = utils::find_widget(right_panel, "export_all");
    let clear: gtk4::Button = utils::find_widget(right_panel, "clear");

    // Exportar selección
    connect_export_selection(&export_selection, table_view, state);
    
    // Exportar todo
    connect_export_all(&export_all, state);
    
    // Limpiar
    connect_clear(&clear, right_panel, state);
}

/// Conecta el evento de exportar selección
fn connect_export_selection(
    button: &gtk4::Button,
    table_view: &gtk4::TreeView,
    state: &Rc<RefCell<AppState>>
) {
    let state_clone = state.clone();
    let tree_view_clone = table_view.clone();
    
    button.connect_clicked(move |_| {
        let selection = tree_view_clone.selection();
        let (paths, model) = selection.selected_rows();

        if !paths.is_empty() {
            let state = state_clone.borrow();
            let mut selected = Vec::new();

            for path in paths {
                if let Some(iter) = model.iter(&path) {
                    if let Ok(strategy) = model.get_value(&iter, 0).get::<String>() {
                        // Buscar la estrategia en los resultados
                        if let Some(result) = state.results.iter()
                            .find(|r| r.strategy == strategy) {
                            selected.push(result.clone());
                        }
                    }
                }
            }

            if !selected.is_empty() {
                let filename = format!(
                    "selection_{}.csv",
                    Local::now().format("%Y%m%d_%H%M%S")
                );
                export_summary_to_csv(&selected, &filename);
                println!("✅ Exportado {} estrategias a {}", selected.len(), filename);
            }
        }
    });
}

/// Conecta el evento de exportar todo
fn connect_export_all(button: &gtk4::Button, state: &Rc<RefCell<AppState>>) {
    let state_clone = state.clone();
    
    button.connect_clicked(move |_| {
        let state = state_clone.borrow();
        if !state.results.is_empty() {
            let filename = format!(
                "all_{}.csv",
                Local::now().format("%Y%m%d_%H%M%S")
            );
            export_summary_to_csv(&state.results, &filename);
            println!(
                "✅ Exportado {} estrategias a {}",
                state.results.len(),
                filename
            );
        }
    });
}

/// Conecta el evento de limpiar
fn connect_clear(
    button: &gtk4::Button,
    right_panel: &gtk4::Box,
    state: &Rc<RefCell<AppState>>
) {
    let state_clone = state.clone();
    let right_panel_clone = right_panel.clone();
    
    button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.clear();
        
        // Actualizar UI
        update_results_count(&right_panel_clone, 0);
        enable_export_buttons(&right_panel_clone, false);
        update_status(&right_panel_clone, "Listo");
    });
}

/// Actualiza el contador de resultados
fn update_results_count(toolbar: &gtk4::Box, count: usize) {
    let label: gtk4::Label = utils::find_widget(toolbar, "results_count");
    label.set_text(&format!("Resultados: {}", count));
}

/// Actualiza el estado en la barra de estado
fn update_status(status_bar: &gtk4::Box, message: &str) {
    let label: gtk4::Label = utils::find_widget(status_bar, "status");
    label.set_text(message);
}

/// Habilita o deshabilita los botones de exportación
fn enable_export_buttons(toolbar: &gtk4::Box, enable: bool) {
    let export_selection: gtk4::Button = utils::find_widget(toolbar, "export_selection");
    let export_all: gtk4::Button = utils::find_widget(toolbar, "export_all");

    export_selection.set_sensitive(enable);
    export_all.set_sensitive(enable);
}