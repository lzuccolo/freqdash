// src/gui/events/filters.rs

use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use glib::value::ToValue;

use crate::gui::state::AppState;
use crate::gui::utils;

/// Conecta los eventos de filtrado con componentes Adwaita
pub fn connect(panel: &gtk4::Box, state: &Rc<RefCell<AppState>>) {
    let search: gtk4::SearchEntry = utils::find_widget(panel, "search");
    
    // Buscar switches en lugar de checkbuttons
    let profit_switch: gtk4::Switch = utils::find_widget(panel, "filter_profit");
    let winrate_switch: gtk4::Switch = utils::find_widget(panel, "filter_winrate");
    let trades_switch: gtk4::Switch = utils::find_widget(panel, "filter_trades");
    let pf_switch: gtk4::Switch = utils::find_widget(panel, "filter_pf");
    let expectancy_switch: gtk4::Switch = utils::find_widget(panel, "filter_expectancy");

    let state_clone = state.clone();

    // Clonar widgets para el closure
    let search_clone = search.clone();
    let profit_switch_clone = profit_switch.clone();
    let winrate_switch_clone = winrate_switch.clone();
    let trades_switch_clone = trades_switch.clone();
    let pf_switch_clone = pf_switch.clone();
    let expectancy_switch_clone = expectancy_switch.clone();

    let apply_filters = move || {
        let search_text = search_clone.text().to_string().to_lowercase();
        let filter_profit = profit_switch_clone.is_active();
        let filter_winrate = winrate_switch_clone.is_active();
        let filter_trades = trades_switch_clone.is_active();
        let filter_pf = pf_switch_clone.is_active();
        let filter_expectancy = expectancy_switch_clone.is_active();

        let state = state_clone.borrow();
        let store = &state.store;

        let mut visible_count = 0;
        let total_count = store.iter_n_children(None);

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
                
                if visible {
                    visible_count += 1;
                }

                if !store.iter_next(&iter) {
                    break;
                }
            }
        }
        
        // Mostrar información sobre filtros aplicados
        println!("Filtros aplicados: {} de {} visibles", visible_count, total_count);
    };

    let filters_clone = Rc::new(apply_filters);

    // Conectar eventos a cada control con debounce para mejor rendimiento
    search.connect_search_changed({
        let filters = filters_clone.clone();
        move |_| {
            // Aplicar filtros directamente sin delay para búsqueda
            filters();
        }
    });

    // Conectar switches con notificación de estado
    profit_switch.connect_state_notify({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    winrate_switch.connect_state_notify({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    trades_switch.connect_state_notify({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    pf_switch.connect_state_notify({
        let filters = filters_clone.clone();
        move |_| filters()
    });
    
    expectancy_switch.connect_state_notify({
        let filters = filters_clone.clone();
        move |_| filters()
    });
}