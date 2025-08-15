// src/gui/events/query.rs

use gtk4::prelude::*;
use gtk4::glib;
use std::rc::Rc;
use std::cell::RefCell;

use crate::gui::state::AppState;
use crate::gui::utils;
use crate::backtest::logic::{get_grid_summary, GridQuery};
use glib::value::ToValue;


/// Conecta los eventos relacionados con las consultas
pub fn connect(
    left_panel: &gtk4::Box,
    right_panel: &gtk4::Box,
    state: &Rc<RefCell<AppState>>
) {
    let execute_button: gtk4::Button = utils::find_widget(left_panel, "execute");
    let progress_bar: gtk4::ProgressBar = utils::find_widget(left_panel, "progress");
    let status_label: gtk4::Label = utils::find_widget(right_panel, "status");
    let spinner: gtk4::Spinner = utils::find_widget(right_panel, "spinner");

    let state_clone = state.clone();
    let left_panel_clone = left_panel.clone();
    let right_panel_clone = right_panel.clone();

    execute_button.connect_clicked(move |button| {
        let mut state = state_clone.borrow_mut();
        if state.is_loading {
            return;
        }

        state.is_loading = true;
        state.clear();

        // UI feedback
        button.set_sensitive(false);
        progress_bar.set_visible(true);
        progress_bar.pulse();
        status_label.set_text("Ejecutando consulta...");
        spinner.set_visible(true);
        spinner.start();

        // Obtener parámetros
        let query = get_query_params(&left_panel_clone);

        // Timer para progress
        let progress_clone = progress_bar.clone();
        let timeout_id = glib::timeout_add_local(
            std::time::Duration::from_millis(100),
            move || {
                progress_clone.pulse();
                glib::ControlFlow::Continue
            }
        );

        let state_clone2 = state_clone.clone();
        let button_clone = button.clone();
        let progress_clone = progress_bar.clone();
        let right_panel_clone2 = right_panel_clone.clone();

        glib::MainContext::default().spawn_local(async move {
            match get_grid_summary(&query).await {
                Ok(mut rows) => {
                    // Ordenar por profit
                    rows.sort_by(|a, b| {
                        b.total_profit
                            .partial_cmp(&a.total_profit)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let mut state = state_clone2.borrow_mut();
                    state.results = rows.clone();

                    // Poblar store
                    populate_store(&state.store, &rows);

                    // Actualizar UI
                    update_results_count(&right_panel_clone2, rows.len());
                    update_status(
                        &right_panel_clone2,
                        &format!("{} estrategias encontradas", rows.len())
                    );
                    enable_export_buttons(&right_panel_clone2, true);

                    state.is_loading = false;
                }
                Err(e) => {
                    update_status(&right_panel_clone2, &format!("Error: {}", e));
                    state_clone2.borrow_mut().is_loading = false;
                }
            }

            // Restaurar UI
            button_clone.set_sensitive(true);
            progress_clone.set_visible(false);
            timeout_id.remove();
            let spinner: gtk4::Spinner = utils::find_widget(&right_panel_clone2, "spinner");
            spinner.stop();
            spinner.set_visible(false);
        });
    });
}

/// Obtiene los parámetros de consulta desde el panel
fn get_query_params(panel: &gtk4::Box) -> GridQuery {
    let exchange: gtk4::ComboBoxText = utils::find_widget(panel, "exchange");
    let currency: gtk4::ComboBoxText = utils::find_widget(panel, "currency");
    let pairlist: gtk4::Entry = utils::find_widget(panel, "pairlist");
    let start_date: gtk4::Entry = utils::find_widget(panel, "start_date");
    let months: gtk4::SpinButton = utils::find_widget(panel, "months");

    GridQuery {
        exchange: exchange
            .active_text()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "BINANCE".to_string()),
        currency: currency
            .active_text()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "USDT".to_string()),
        pairlist: pairlist.text().to_string(),
        start_date: start_date.text().to_string(),
        months: months.value() as usize,
    }
}

/// Pobla el store con los resultados
fn populate_store(store: &gtk4::ListStore, rows: &[crate::backtest::model::StrategyGridRow]) {
    
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
                (7, &r.trailing_stop_positive_offset.unwrap_or(0.0).to_value()),
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
                (30, &true.to_value()), // Visible por defecto
            ],
        );
    }
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