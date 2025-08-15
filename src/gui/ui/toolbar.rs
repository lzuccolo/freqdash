// src/gui_adw/ui/toolbar.rs

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita::HeaderBar;

pub fn create() -> HeaderBar {
    let header_bar = HeaderBar::new();
    header_bar.add_css_class("flat");

    let menu_button = Button::builder()
        .name("flap_toggle") // Give it a name to connect the event
        .icon_name("sidebar-show-symbolic")
        .build();
    header_bar.pack_start(&menu_button);

    let results_label = Label::builder()
        .name("results_count")
        .halign(gtk4::Align::Start)
        .build();
    results_label.set_markup("<b>Resultados:</b> 0");
    header_bar.pack_start(&results_label);

    let export_selection = Button::builder()
        .name("export_selection")
        .icon_name("document-save-symbolic")
        .tooltip_text("Exportar Selección")
        .sensitive(false)
        .build();
    header_bar.pack_end(&export_selection);

    let export_all = Button::builder()
        .name("export_all")
        .icon_name("document-save-as-symbolic")
        .tooltip_text("Exportar Todo")
        .sensitive(false)
        .build();
    header_bar.pack_end(&export_all);

    let clear_button = Button::builder()
        .name("clear")
        .icon_name("edit-clear-all-symbolic")
        .tooltip_text("Limpiar")
        .build();
    header_bar.pack_end(&clear_button);

    header_bar
}
