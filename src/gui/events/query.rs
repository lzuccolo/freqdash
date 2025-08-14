// src/gui_adw/events/query.rs

use gtk4::prelude::*;
use gtk4::{
    glib, Box as GtkBox, Button, Label, ListStore, ProgressBar, Spinner, TreeModelFilter, TreeView,
};
use libadwaita::ComboRow;
use libadwaita::HeaderBar;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use tokio::runtime::Runtime;

use crate::backtest::logic::get_grid_summary;
use crate::backtest::model::{GridQuery, StrategyGridRow};
use crate::gui_adw::app::{get_runtime, DatabaseCommand};
use crate::gui_adw::state::AppState;
use crate::gui_adw::utils;
use glib::value::ToValue;

pub const BATCH_SIZE: usize = 200;

pub fn connect(
    left_panel: &GtkBox,
    right_panel: &GtkBox,
    header_bar: &HeaderBar,
    tree_view: &TreeView,
    state: &Rc<RefCell<AppState>>,
    command_tx: mpsc::Sender<DatabaseCommand>,
) {
    let execute_button: Button = utils::find_widget(left_panel, "execute");

    let tree_view_clone = tree_view.clone();
    let state_clone = state.clone();
    let left_panel_clone = left_panel.clone();
    let right_panel_clone = right_panel.clone();
    let header_bar_clone = header_bar.clone();

    execute_button.connect_clicked(move |_button| {
        if state_clone.borrow().is_loading {
            return;
        }

        let mut state = state_clone.borrow_mut();
        state.is_loading = true;

        let column_types: Vec<glib::Type> = (0..state.store.n_columns())
            .map(|i| state.store.column_type(i))
            .collect();
        let new_store = ListStore::new(&column_types);
        let new_filter_model = TreeModelFilter::new(&new_store, None);
        new_filter_model.set_visible_column(30);
        tree_view_clone.set_model(Some(&new_filter_model));
        state.store = new_store;
        state.filter_model = new_filter_model;
        state.results.clear();
        drop(state);

        let execute_button: Button = utils::find_widget(&left_panel_clone, "execute");
        let progress_bar: ProgressBar = utils::find_widget(&left_panel_clone, "progress");
        let status_label: Label = utils::find_widget(&right_panel_clone, "status");
        let spinner: Spinner = utils::find_widget(&right_panel_clone, "spinner");

        execute_button.set_sensitive(false);
        progress_bar.set_visible(true);
        status_label.set_text("Enviando consulta al trabajador...");
        spinner.set_visible(true);
        spinner.start();
        enable_export_buttons(&header_bar_clone, false);

        let query = get_query_params(&left_panel_clone);
        command_tx
            .send(DatabaseCommand::RunBacktest(query))
            .expect("Failed to send command");
    });
}

pub fn get_query_params(panel: &GtkBox) -> GridQuery {
    let exchange: ComboRow = utils::find_widget(panel, "exchange");
    let currency: ComboRow = utils::find_widget(panel, "currency");
    let exchange_text = utils::get_combo_row_text(&exchange);
    let currency_text = utils::get_combo_row_text(&currency);
    let pairlist_combo: ComboRow = utils::find_widget(panel, "pairlist_combo");    
    let pairlist_text = utils::get_combo_row_text(&pairlist_combo);  
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
        pairlist: pairlist_text,
        start_date: start_date.text().to_string(),
        months: months.value() as usize,
    }
}

pub fn populate_store_batch(store: &ListStore, batch: &[StrategyGridRow]) {
    for r in batch {
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

pub fn update_results_count(toolbar: &HeaderBar, count: usize) {
    let label: Label = utils::find_widget(toolbar, "results_count");
    label.set_markup(&format!("<b>Resultados:</b> {}", count));
}

pub fn update_status(status_bar: &GtkBox, message: &str) {
    let label: Label = utils::find_widget(status_bar, "status");
    label.set_text(message);
}

pub fn enable_export_buttons(toolbar: &HeaderBar, enable: bool) {
    let export_selection: Button = utils::find_widget(toolbar, "export_selection");
    let export_all: Button = utils::find_widget(toolbar, "export_all");
    let clear: Button = utils::find_widget(toolbar, "clear");
    export_selection.set_sensitive(enable);
    export_all.set_sensitive(enable);
    clear.set_sensitive(enable);
}
