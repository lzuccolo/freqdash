use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Spinner};

/// Crea la barra de estado con indicadores de actividad
pub fn create() -> Box {
    let status_bar = Box::new(Orientation::Horizontal, 8);
    status_bar.set_margin_top(8);
    status_bar.set_margin_bottom(8);
    status_bar.set_margin_start(8);
    status_bar.set_margin_end(8);

    // Label de estado
    let status_label = Label::new(Some("Listo"));
    status_label.set_widget_name("status");
    status_bar.append(&status_label);

    // Spinner para indicar actividad
    let spinner = Spinner::new();
    spinner.set_widget_name("spinner");
    spinner.set_visible(false);
    status_bar.append(&spinner);

    status_bar
}