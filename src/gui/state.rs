use gtk4::ListStore;
use std::cell::RefCell;
use std::rc::Rc;
use crate::backtest::model::StrategyGridRow;

/// Estado compartido de la aplicación
pub struct AppState {
    pub store: ListStore,
    pub results: Vec<StrategyGridRow>,
    pub is_loading: bool,
}

impl AppState {
    /// Crea una nueva instancia del estado de la aplicación
    pub fn new(store: ListStore) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            store,
            results: Vec::new(),
            is_loading: false,
        }))
    }
    
    /// Limpia todos los datos
    pub fn clear(&mut self) {
        self.store.clear();
        self.results.clear();
        self.is_loading = false;
    }
    
    /// Verifica si hay resultados
    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }
    
    /// Obtiene el número de resultados
    pub fn results_count(&self) -> usize {
        self.results.len()
    }
}