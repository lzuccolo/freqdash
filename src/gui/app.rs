use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Orientation, Separator};

use crate::gui::state::AppState;
use crate::gui::ui;
use crate::gui::events;

/// Lanza la aplicación GTK
pub fn launch() {
    let app = Application::builder()
        .application_id("com.example.freqdash.gtk4")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

/// Construye la interfaz de usuario principal
fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Freqdash - Análisis de Backtests")
        .default_width(1600)
        .default_height(800)
        .build();

    // Crear el layout principal
    let main_box = Box::new(Orientation::Horizontal, 0);
    
    // Crear panel izquierdo
    let left_panel = ui::left_panel::create();
    left_panel.set_size_request(350, -1);
    
    // Crear panel derecho con todos sus componentes
    let (right_panel, table_view, filter_model) = ui::create_right_panel();
    
    // Ensamblar la UI
    main_box.append(&left_panel);
    main_box.append(&Separator::new(Orientation::Vertical));
    main_box.append(&right_panel);
    
    // Inicializar estado con el store del filter_model
    let store = ui::table_view::get_base_store(&filter_model);
    let state = AppState::new(store);
    
    // Conectar todos los eventos
    events::connect_all(&left_panel, &right_panel, &table_view, &state);
    
    window.set_child(Some(&main_box));
    window.show();
}