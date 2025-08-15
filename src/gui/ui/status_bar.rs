// src/gui/ui/status_bar.rs

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Spinner};
use libadwaita::StatusPage;

/// Crea la barra de estado con indicadores de actividad estilo Adwaita
pub fn create() -> Box {
    let status_bar = Box::new(Orientation::Horizontal, 12);
    status_bar.add_css_class("dim-label");
    status_bar.set_margin_top(8);
    status_bar.set_margin_bottom(8);
    status_bar.set_margin_start(12);
    status_bar.set_margin_end(12);

    // Label de estado con estilo
    let status_label = Label::new(Some("Listo"));
    status_label.set_widget_name("status");
    status_label.add_css_class("body");
    status_bar.append(&status_label);

    // Spinner con estilo Adwaita
    let spinner = Spinner::new();
    spinner.set_widget_name("spinner");
    spinner.set_visible(false);
    spinner.add_css_class("thick");
    status_bar.append(&spinner);

    status_bar
}

/// Crea una página de estado vacía (para cuando no hay datos)
pub fn create_empty_state() -> StatusPage {
    let status_page = StatusPage::new();
    status_page.set_icon_name(Some("document-open-symbolic"));
    status_page.set_title("No hay datos");
    status_page.set_description(Some("Configure los parámetros y ejecute una consulta para ver los resultados"));
    
    status_page
}