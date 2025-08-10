use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};

/// Crea la barra de herramientas con controles de resultados
pub fn create() -> Box {
    let toolbar = Box::new(Orientation::Horizontal, 8);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);

    // Label de contador de resultados
    let results_label = Label::new(Some("Resultados: 0"));
    results_label.set_widget_name("results_count");
    toolbar.append(&results_label);

    // Spacer para empujar botones a la derecha
    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    // Botón exportar selección
    let export_button = Button::with_label("Exportar Selección");
    export_button.set_widget_name("export_selection");
    export_button.set_sensitive(false);
    toolbar.append(&export_button);

    // Botón exportar todo
    let export_all_button = Button::with_label("Exportar Todo");
    export_all_button.set_widget_name("export_all");
    export_all_button.set_sensitive(false);
    toolbar.append(&export_all_button);

    // Botón limpiar
    let clear_button = Button::with_label("Limpiar");
    clear_button.set_widget_name("clear");
    toolbar.append(&clear_button);

    toolbar
}