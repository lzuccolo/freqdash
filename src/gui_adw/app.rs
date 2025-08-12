// src/gui_adw/app.rs

use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk4::{Box, Orientation, Separator};
use std::sync::Arc;
use tokio::runtime::Runtime;
use std::cell::RefCell;
use std::rc::Rc;

use crate::gui_adw::state::AppState;
use crate::gui_adw::ui;
use crate::gui_adw::events;

thread_local! {
    static RUNTIME: RefCell<Option<Arc<Runtime>>> = RefCell::new(None);
}

/// Obtiene el runtime de Tokio almacenado
pub fn get_runtime() -> Arc<Runtime> {
    RUNTIME.with(|rt| {
        rt.borrow()
            .as_ref()
            .expect("Runtime not initialized")
            .clone()
    })
}

/// Lanza la aplicación Adwaita con runtime de Tokio
pub fn launch(rt: Arc<Runtime>) {
    // Guardar el runtime en thread-local storage
    RUNTIME.with(|runtime| {
        *runtime.borrow_mut() = Some(rt.clone());
    });
    
    let app = Application::builder()
        .application_id("com.example.freqdash.adwaita")
        .build();

    app.connect_activate(move |app| {
        build_ui(app);
    });
    
    app.run();
}

/// Construye la interfaz de usuario principal
fn build_ui(app: &Application) {
    // Ventana principal con estilo Adwaita
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Freqdash - Análisis de Backtests")
        .default_width(1600)
        .default_height(900)
        .build();
    
    // Crear header bar nativo de Adwaita
    let header_bar = HeaderBar::builder()
        .title_widget(&libadwaita::WindowTitle::new("Freqdash", "Análisis de Backtests"))
        .build();
    
    // Layout principal usando ToolbarView de Adwaita
    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);
    
    // Contenedor principal
    let main_box = Box::new(Orientation::Horizontal, 0);
    
    // Panel izquierdo con estilo Adwaita
    let left_panel = ui::left_panel::create();
    left_panel.set_size_request(380, -1);
    
    // Panel derecho
    let (right_panel, table_view, filter_model) = ui::create_right_panel();
    
    // Ensamblar la UI
    main_box.append(&left_panel);
    main_box.append(&Separator::new(Orientation::Vertical));
    main_box.append(&right_panel);
    
    // Configurar el contenido del ToolbarView
    toolbar_view.set_content(Some(&main_box));
    
    // Inicializar estado
    let store = ui::table_view::get_base_store(&filter_model);
    let state = AppState::new(store);
    
    // Conectar eventos (pasando el runtime implícitamente a través del thread-local)
    events::connect_all(&left_panel, &right_panel, &table_view, &state);
    
    // Configurar ventana
    window.set_content(Some(&toolbar_view));
    window.present();
}