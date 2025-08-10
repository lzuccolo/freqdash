// src/gui/events/filters.rs

use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use glib::value::ToValue;


use crate::gui::state::AppState;
use crate::gui::utils;

/// Conecta los eventos de filtrado
pub fn connect(panel: &gtk4::Box, state: &Rc<RefCell<AppState>>) {
    let search: gtk4::SearchEntry = utils::find_widget(panel, "search");
    let profit_check: gtk4::CheckButton = utils::find_widget(panel, "filter_profit");
    let winrate_check: gtk4::CheckButton = utils::find_widget(panel, "filter_winrate");
    let trades_check: gtk4::CheckButton = utils::find_widget(panel, "filter_trades");
    let pf_check: gtk4::CheckButton = utils::find_widget(panel, "filter_pf");
    let expectancy_check: gtk4::CheckButton = utils::find_widget(panel, "filter_expectancy");

    let state_clone = state.clone();

    // Clonar widgets para el closure
    let search_clone = search.clone();
    let profit_check_clone = profit_check.clone();
    let winrate_check_clone = winrate_check.clone();
    let trades_check_clone = trades_check.clone();
    let pf_check_clone = pf_check.clone();
    let expectancy_check_clone = expectancy_check.clone();

    let apply_filters = move || {
        let search_text = search_clone.text().to_string().to_lowercase();
        let filter_profit = profit_check_clone.is_active();
        let filter_winrate = winrate_check_clone.is_active();
        let filter_trades = trades_check_clone.is_active();
        let filter_pf = pf_check_clone.is_active();
        let filter_expectancy = expectancy_check_clone.is_active();

        let state = state_clone.borrow();
        let store = &state.store;

        if let Some(iter) = store.iter_first() {
            loop {
                let mut visible = true;

                // Filtro de búsqueda por texto
                if !search_text.is_empty() {
                    if let Ok(strategy) = store.get_value(&iter, 29).get::<String>() {
                        if !strategy.contains(&search_text) {
                            visible = false;
                        }
                    }
                }

                // Filtro por profit positivo
                if visible && filter_profit {
                    if let Ok(profit) = store.get_value(&iter, 12).get::<f64>() {
                        if profit <= 0.0 {
                            visible = false;
                        }
                    }
                }

                // Filtro por win rate
                if visible && filter_winrate {
                    if let Ok(winrate) = store.get_value(&iter, 15).get::<f64>() {
                        if winrate <= 0.5 {
                            visible = false;
                        }
                    }
                }

                // Filtro por número de trades
                if visible && filter_trades {
                    if let Ok(trades) = store.get_value(&iter, 13).get::<i32>() {
                        if trades <= 100 {
                            visible = false;
                        }
                    }
                }

                // Filtro por profit factor
                if visible && filter_pf {
                    if let Ok(pf) = store.get_value(&iter, 28).get::<f64>() {
                        if pf <= 1.0 {
                            visible = false;
                        }
                    }
                }
                
                // Filtro por expectancy
                if visible && filter_expectancy {
                    if let Ok(expectancy) = store.get_value(&iter, 27).get::<f64>() {
                        if expectancy <= 0.0 {
                            visible = false;
                        }
                    }
                }

                // Actualizar visibilidad en el store
                store.set_value(&iter, 30, &visible.to_value());

                if !store.iter_next(&iter) {
                    break;
                }
            }
        }
    };

    let filters_clone = Rc::new(apply_filters);

    // Conectar eventos a cada control
    search.connect_search_changed({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    profit_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    winrate_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    trades_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    pf_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });
    
    expectancy_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });
}