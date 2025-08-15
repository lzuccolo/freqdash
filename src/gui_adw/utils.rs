// src/gui_adw/utils.rs

use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::ComboRow;
use libadwaita::ActionRow;


/// Busca un widget por su nombre dentro de un contenedor
pub fn find_widget<T: IsA<gtk4::Widget>>(parent: &impl IsA<gtk4::Widget>, name: &str) -> T {
    let mut queue = vec![parent.clone().upcast::<gtk4::Widget>()];

    while let Some(widget) = queue.pop() {
        if widget.widget_name() == name {
            return widget
                .downcast::<T>()
                .expect(&format!("Widget '{}' not found or wrong type", name));
        }

        // Buscar en los hijos
        let mut child = widget.first_child();
        while let Some(c) = child {
            queue.push(c.clone());
            child = c.next_sibling();
        }
    }

    panic!("Widget '{}' not found", name);
}

/// Busca un widget de forma segura, retornando Option
pub fn find_widget_safe<T: IsA<gtk4::Widget>>(
    parent: &impl IsA<gtk4::Widget>,
    name: &str
) -> Option<T> {
    let mut queue = vec![parent.clone().upcast::<gtk4::Widget>()];

    while let Some(widget) = queue.pop() {
        if widget.widget_name() == name {
            return widget.downcast::<T>().ok();
        }

        let mut child = widget.first_child();
        while let Some(c) = child {
            queue.push(c.clone());
            child = c.next_sibling();
        }
    }

    None
}

/// Obtiene el valor seleccionado de un ComboRow de Adwaita
pub fn get_combo_row_text(combo_row: &ComboRow) -> String {
    if let Some(model) = combo_row.model() {
        if let Ok(string_list) = model.downcast::<gtk4::StringList>() {
            let selected = combo_row.selected();
            if let Some(string) = string_list.string(selected) {
                return string.to_string();
            }
        }
    }
    String::new()
}

/// Obtiene el Entry de un ActionRow
pub fn get_action_row_entry(action_row: &ActionRow) -> Option<gtk4::Entry> {
    let mut child = action_row.first_child();
    while let Some(widget) = child {
        if let Ok(entry) = widget.clone().downcast::<gtk4::Entry>() {
            return Some(entry);
        }
        child = widget.next_sibling();
    }
    None
}

/// Obtiene el Switch de un ActionRow
pub fn get_action_row_switch(action_row: &ActionRow) -> Option<gtk4::Switch> {
    let mut child = action_row.first_child();
    while let Some(widget) = child {
        if let Ok(switch) = widget.clone().downcast::<gtk4::Switch>() {
            return Some(switch);
        }
        child = widget.next_sibling();
    }
    None
}