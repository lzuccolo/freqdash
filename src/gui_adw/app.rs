// src/gui_adw/app.rs

use adw::prelude::*;
use gtk4::prelude::*;
use libadwaita as adw;
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Runtime;

use crate::gui_adw::state::AppState;
use crate::gui_adw::{events, ui}; // Importamos los módulos completos

// --- GESTIÓN DEL RUNTIME DE TOKIO ---
static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME
        .get()
        .expect("El runtime de Tokio no está inicializado")
}

// --- LÓGICA DE LA APLICACIÓN GTK ---
pub fn run() {
    // Inicializamos el runtime de Tokio una sola vez
    TOKIO_RUNTIME
        .set(Runtime::new().expect("No se pudo crear el runtime de Tokio"))
        .expect("El runtime de Tokio ya estaba inicializado");
    println!("✅ AsyncDatabase inicializada correctamente.");

    let app = adw::Application::new(Some("com.example.freqdash"), Default::default());
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    // Crear paneles
    let left_panel = ui::left_panel::create();
    let (right_panel, tree_view, filter_model) = ui::right_panel::create();

    content.append(&left_panel);
    content.append(&right_panel);

    // Crear el estado inicial, obteniendo el ListStore desde el TreeModelFilter
    let store = ui::table_view::get_base_store(&filter_model);
    let app_state = Rc::new(RefCell::new(AppState::new(store, filter_model)));

    // Conectar eventos, pasando los componentes necesarios
    events::connect_all(
        &left_panel,
        &right_panel,
        &tree_view, // Pasamos el TreeView para poder reemplazar su modelo
        &app_state,
    );

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("FreqDash")
        .default_width(1200)
        .default_height(800)
        .content(&content)
        .build();

    window.present();
}
