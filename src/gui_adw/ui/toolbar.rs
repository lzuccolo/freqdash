// src/gui_adw/ui/toolbar.rs

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita::ButtonContent;

/// Crea la barra de herramientas con controles de resultados estilo Adwaita
pub fn create() -> Box {
    let toolbar = Box::new(Orientation::Horizontal, 12);
    toolbar.add_css_class("toolbar");
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);
    toolbar.set_margin_start(12);
    toolbar.set_margin_end(12);

    // Label de contador con estilo
    let results_label = Label::new(Some("Resultados: 0"));
    results_label.set_widget_name("results_count");
    results_label.add_css_class("title-4");
    toolbar.append(&results_label);

    // Spacer
    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    // Botón exportar selección con icono
    let export_button = Button::new();
    let export_content = ButtonContent::new();
    export_content.set_label("Exportar Selección");
    export_content.set_icon_name("document-save-symbolic");
    export_button.set_child(Some(&export_content));
    export_button.set_widget_name("export_selection");
    export_button.set_sensitive(false);
    export_button.add_css_class("flat");
    toolbar.append(&export_button);

    // Botón exportar todo con icono
    let export_all_button = Button::new();
    let export_all_content = ButtonContent::new();
    export_all_content.set_label("Exportar Todo");
    export_all_content.set_icon_name("document-save-as-symbolic");
    export_all_button.set_child(Some(&export_all_content));
    export_all_button.set_widget_name("export_all");
    export_all_button.set_sensitive(false);
    export_all_button.add_css_class("flat");
    toolbar.append(&export_all_button);

    // Botón limpiar con icono y estilo destructivo
    let clear_button = Button::new();
    let clear_content = ButtonContent::new();
    clear_content.set_label("Limpiar");
    clear_content.set_icon_name("user-trash-symbolic");
    clear_button.set_child(Some(&clear_content));
    clear_button.set_widget_name("clear");
    clear_button.add_css_class("flat");
    toolbar.append(&clear_button);

    toolbar
}