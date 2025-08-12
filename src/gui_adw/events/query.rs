// src/gui_adw/events/query.rs

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::ColumnView;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use tokio::runtime::Runtime;

use libadwaita::ComboRow;

use crate::backtest::logic::{get_grid_summary, GridQuery};
use crate::backtest::model::StrategyGridRow;
use crate::gui_adw::app::get_runtime;
use crate::gui_adw::state::AppState;
use crate::gui_adw::utils;
use glib::value::ToValue; // Asumiendo que esta es la ruta a tu runtime de Tokio

/// Conecta los eventos relacionados con las consultas
pub fn connect(left_panel: &gtk4::Box, right_panel: &gtk4::Box, state: &Rc<RefCell<AppState>>) {
    let execute_button: gtk4::Button = utils::find_widget(left_panel, "execute");
    let progress_bar: gtk4::ProgressBar = utils::find_widget(left_panel, "progress");
    let status_label: gtk4::Label = utils::find_widget(right_panel, "status");
    let spinner: gtk4::Spinner = utils::find_widget(right_panel, "spinner");

    let state_clone = state.clone();
    let left_panel_clone = left_panel.clone();
    let right_panel_clone = right_panel.clone();

    execute_button.connect_clicked(move |button| {
        if state_clone.borrow().is_loading {
            return;
        }

        let mut state = state_clone.borrow_mut();
        state.is_loading = true;

        // --- LA SOLUCIÓN: REEMPLAZAR EL MODELO ---
        // 1. Obtenemos los tipos de las columnas del store viejo para crear uno nuevo idéntico.
        let column_types: Vec<glib::Type> = (0..state.store.n_columns())
            .map(|i| state.store.column_type(i))
            .collect();

        // 2. Creamos un nuevo store, completamente vacío.
        let new_store = gtk4::ListStore::new(&column_types);

        // 3. Le decimos a la vista que use el nuevo store. ¡Esto es instantáneo!
        results_view.set_model(Some(&new_store));

        // 4. Actualizamos el estado de la aplicación para que apunte al nuevo store.
        state.store = new_store;
        state.results.clear();
        drop(state); // Liberamos el RefCell

        //state.clear();
        //drop(state);

        // --- 1. PREPARAR UI PARA CARGA ---
        button.set_sensitive(false);
        button.add_css_class("flat");
        progress_bar.set_visible(true);
        progress_bar.add_css_class("osd");
        status_label.set_text("Ejecutando consulta...");
        spinner.set_visible(true);
        spinner.start();
        enable_export_buttons(&right_panel_clone, false);

        // --- 2. CREAR CANAL DE COMUNICACIÓN ---
        let (tx, rx) = mpsc::channel::<Result<Vec<StrategyGridRow>, String>>();

        let query = get_query_params(&left_panel_clone);

        // --- 3. LANZAR TAREA PESADA A OTRO HILO ---
        thread::spawn(move || {
            /*  // Obtenemos el runtime de Tokio para ejecutar la tarea asíncrona
            let rt = get_runtime();
            // Usamos block_on porque estamos dentro de un hilo síncrono normal
            let result = rt.block_on(async {
                // Ahora usamos .await DENTRO del bloque asíncrono
                get_grid_summary(&query).await.map_err(|e| e.to_string())
            }); */

            // Creamos un nuevo runtime de Tokio solo para este hilo.
            let rt = Runtime::new().expect("No se pudo crear el runtime de Tokio");
            // Usamos block_on para ejecutar la tarea y esperar su resultado.
            let result = rt
                .block_on(get_grid_summary(&query))
                .map_err(|e| e.to_string());

            let _ = tx.send(result);
        });

        let state_clone2 = state_clone.clone();
        let button_clone = button.clone();
        let progress_clone = progress_bar.clone();
        let right_panel_clone2 = right_panel_clone.clone();
        let spinner_clone = spinner.clone();

        // --- 4. ESPERAR RESULTADO EN HILO PRINCIPAL SIN BLOQUEAR ---
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            match rx.try_recv() {
                Ok(result) => {
                    // --- 5. ACTUALIZAR UI CON EL RESULTADO ---
                    match result {
                        Ok(mut rows) => {
                            rows.sort_by(|a, b| {
                                b.total_profit
                                    .partial_cmp(&a.total_profit)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });

                            let mut state = state_clone2.borrow_mut();
                            state.results = rows.clone();

                            populate_store(&state.store, &rows);

                            update_results_count(&right_panel_clone2, rows.len());
                            update_status(
                                &right_panel_clone2,
                                &format!("✅ {} estrategias encontradas", rows.len()),
                            );
                            enable_export_buttons(&right_panel_clone2, !rows.is_empty());
                        }
                        Err(e_string) => {
                            update_status(&right_panel_clone2, &format!("❌ Error: {}", e_string));
                        }
                    }

                    let mut state = state_clone2.borrow_mut();
                    state.is_loading = false;

                    button_clone.set_sensitive(true);
                    button_clone.remove_css_class("flat");
                    progress_clone.set_visible(false);
                    spinner_clone.stop();
                    spinner_clone.set_visible(false);

                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => {
                    progress_clone.pulse();
                    glib::ControlFlow::Continue
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    update_status(&right_panel_clone2, "❌ Error: El hilo de trabajo falló.");
                    glib::ControlFlow::Break
                }
            }
        });
    });
}

/// Obtiene los parámetros de consulta desde el panel con componentes Adwaita
fn get_query_params(panel: &gtk4::Box) -> GridQuery {
    let exchange: ComboRow = utils::find_widget(panel, "exchange");
    let currency: ComboRow = utils::find_widget(panel, "currency");

    let exchange_text = utils::get_combo_row_text(&exchange);
    let currency_text = utils::get_combo_row_text(&currency);

    let pairlist: gtk4::Entry = utils::find_widget(panel, "pairlist");
    let start_date: gtk4::Entry = utils::find_widget(panel, "start_date");
    let months: gtk4::SpinButton = utils::find_widget(panel, "months");

    GridQuery {
        exchange: if exchange_text.is_empty() {
            "BINANCE".to_string()
        } else {
            exchange_text
        },
        currency: if currency_text.is_empty() {
            "USDT".to_string()
        } else {
            currency_text
        },
        pairlist: pairlist.text().to_string(),
        start_date: start_date.text().to_string(),
        months: months.value() as usize,
    }
}

/// Pobla el store con los resultados
fn populate_store(store: &gtk4::ListStore, rows: &[StrategyGridRow]) {
    store.clear();
    for r in rows {
        store.insert_with_values(
            None,
            &[
                (0, &r.strategy.to_value()),
                (1, &r.timeframe.to_value()),
                (2, &r.minimal_roi.to_value()),
                (3, &r.stoploss.parse::<f64>().unwrap_or(-99.0).to_value()),
                (4, &r.max_open_trades.to_value()),
                (5, &r.trailing_stop.to_value()),
                (6, &r.trailing_stop_positive.unwrap_or(0.0).to_value()),
                (
                    7,
                    &r.trailing_stop_positive_offset.unwrap_or(0.0).to_value(),
                ),
                (8, &r.trailing_only_offset_is_reached.to_value()),
                (9, &r.entry_price.to_value()),
                (10, &r.exit_price.to_value()),
                (11, &r.check_depth_of_market_enable.to_value()),
                (12, &r.total_profit.to_value()),
                (13, &r.total_trades.to_value()),
                (14, &r.wins.to_value()),
                (15, &r.win_rate.to_value()),
                (16, &r.win_time.to_value()),
                (17, &r.drawdown_perc.to_value()),
                (18, &(r.rejected_signals as i32).to_value()),
                (19, &(r.neg_months as i32).to_value()),
                (20, &r.avg_monthly_profit.to_value()),
                (21, &r.std_monthly_profit.to_value()),
                (22, &r.max_profit_month.to_value()),
                (23, &r.min_profit_month.to_value()),
                (24, &r.avg_trade_profit.to_value()),
                (25, &r.losses.to_value()),
                (26, &r.loss_rate.to_value()),
                (27, &r.expectancy.to_value()),
                (28, &r.profit_factor.to_value()),
                (29, &r.strategy.to_lowercase().to_value()),
                (30, &true.to_value()),
            ],
        );
    }
}

/// Actualiza el contador de resultados con estilo
fn update_results_count(toolbar: &gtk4::Box, count: usize) {
    let label: gtk4::Label = utils::find_widget(toolbar, "results_count");
    label.set_markup(&format!("<b>Resultados:</b> {}", count));
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

    if enable {
        export_selection.remove_css_class("flat");
        export_all.remove_css_class("flat");
    } else {
        export_selection.add_css_class("flat");
        export_all.add_css_class("flat");
    }
}
