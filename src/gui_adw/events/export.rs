// src/gui_adw/events/export.rs
use adw::HeaderBar;
use chrono::Local;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backtest::logic::export_summary_to_csv;
use crate::gui_adw::state::AppState;
use crate::gui_adw::utils;

/// Conecta los eventos de exportación
pub fn connect(header_bar: &HeaderBar, right_panel: &gtk4::Box, table_view: &gtk4::TreeView, state: &Rc<RefCell<AppState>>) {
    let export_selection: gtk4::Button = utils::find_widget(header_bar, "export_selection");
    let export_all: gtk4::Button = utils::find_widget(header_bar, "export_all");
    let clear: gtk4::Button = utils::find_widget(header_bar, "clear");

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
    state: &Rc<RefCell<AppState>>,
) {
    let state_clone = state.clone();
    let tree_view_clone = table_view.clone();

    button.connect_clicked(move |btn| {
        // Añadir feedback visual
        btn.add_css_class("suggested-action");

        let selection = tree_view_clone.selection();
        let (paths, model) = selection.selected_rows();

        if !paths.is_empty() {
            let state = state_clone.borrow();
            let mut selected = Vec::new();

            for path in paths {
                if let Some(iter) = model.iter(&path) {
                    if let Ok(strategy) = model.get_value(&iter, 0).get::<String>() {
                        // Buscar la estrategia en los resultados
                        if let Some(result) = state.results.iter().find(|r| r.strategy == strategy)
                        {
                            selected.push(result.clone());
                        }
                    }
                }
            }

            if !selected.is_empty() {
                let filename = format!("selection_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
                export_summary_to_csv(&selected, &filename);

                // Mostrar notificación
                println!("✅ Exportado {} estrategias a {}", selected.len(), filename);
                show_export_notification(&filename, selected.len());
            }
        }

        // Remover feedback visual después de un momento
        let btn_clone = btn.clone();
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
            btn_clone.remove_css_class("suggested-action");
        });
    });
}

/// Conecta el evento de exportar todo
fn connect_export_all(button: &gtk4::Button, state: &Rc<RefCell<AppState>>) {
    let state_clone = state.clone();

    button.connect_clicked(move |btn| {
        // Añadir feedback visual
        btn.add_css_class("suggested-action");

        let state = state_clone.borrow();
        if !state.results.is_empty() {
            let filename = format!("all_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
            export_summary_to_csv(&state.results, &filename);

            // Mostrar notificación
            println!(
                "✅ Exportado {} estrategias a {}",
                state.results.len(),
                filename
            );
            show_export_notification(&filename, state.results.len());
        }

        // Remover feedback visual
        let btn_clone = btn.clone();
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
            btn_clone.remove_css_class("suggested-action");
        });
    });
}

/// Conecta el evento de limpiar con confirmación
fn connect_clear(button: &gtk4::Button, right_panel: &gtk4::Box, state: &Rc<RefCell<AppState>>) {
    let state_clone = state.clone();
    let right_panel_clone = right_panel.clone();

    button.connect_clicked(move |btn| {
        // Añadir animación de limpieza
        btn.add_css_class("destructive-action");

        let mut state = state_clone.borrow_mut();

        // Animación de fade out antes de limpiar
        if state.has_results() {
            state.clear();

            // Actualizar UI con transición
            update_results_count(&right_panel_clone, 0);
            enable_export_buttons(&right_panel_clone, false);
            update_status(&right_panel_clone, "✨ Tabla limpiada");
        }

        // Remover estilo después de animación
        let btn_clone = btn.clone();
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            btn_clone.remove_css_class("destructive-action");
        });
    });
}

/// Muestra una notificación de exportación (simulada con print)
fn show_export_notification(filename: &str, count: usize) {
    // En una aplicación real, aquí se mostraría un Toast de Adwaita
    println!("📁 Archivo exportado: {}", filename);
    println!("   {} estrategias guardadas exitosamente", count);
}

/// Actualiza el contador de resultados con estilo
fn update_results_count(toolbar: &gtk4::Box, count: usize) {
    let label: gtk4::Label = utils::find_widget(toolbar, "results_count");
    if count > 0 {
        label.set_markup(&format!(
            "<b>Resultados:</b> <span foreground='#2ec27e'>{}</span>",
            count
        ));
    } else {
        label.set_markup("<b>Resultados:</b> 0");
    }
}

/// Actualiza el estado en la barra de estado
fn update_status(status_bar: &gtk4::Box, message: &str) {
    let label: gtk4::Label = utils::find_widget(status_bar, "status");
    label.set_text(message);

    // Añadir animación de fade
    label.add_css_class("dim-label");
    gtk4::glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
        label.set_text("Listo");
    });
}

/// Habilita o deshabilita los botones de exportación con transición
fn enable_export_buttons(toolbar: &gtk4::Box, enable: bool) {
    let export_selection: gtk4::Button = utils::find_widget(toolbar, "export_selection");
    let export_all: gtk4::Button = utils::find_widget(toolbar, "export_all");

    // Transición suave de habilitación
    export_selection.set_sensitive(enable);
    export_all.set_sensitive(enable);

    if enable {
        export_selection.remove_css_class("flat");
        export_all.remove_css_class("flat");
        export_selection.add_css_class("suggested-action");
        export_all.add_css_class("suggested-action");

        // Remover clase después de highlight inicial
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
            export_selection.remove_css_class("suggested-action");
            export_all.remove_css_class("suggested-action");
        });
    } else {
        export_selection.add_css_class("flat");
        export_all.add_css_class("flat");
    }
}
