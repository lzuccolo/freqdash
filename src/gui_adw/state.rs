// src/gui_adw/state.rs

use gtk4::{TreeModelFilter, ListStore};
use crate::backtest::model::StrategyGridRow;

#[derive(Clone)]
pub struct AppState {
    pub is_loading: bool,
    pub results: Vec<StrategyGridRow>,
    pub store: ListStore,
    pub filter_model: TreeModelFilter,
}

impl AppState {
    pub fn new(store: ListStore, filter_model: TreeModelFilter) -> Self { // <--- MODIFICAR CONSTRUCTOR
        Self {
            is_loading: false,
            results: Vec::new(),
            store,
            filter_model,
        }
    }

    pub fn clear(&mut self) {
        self.results.clear();
    }

    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub fn results_count(&self) -> usize {
        self.results.len()
    }
}