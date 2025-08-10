// src/gui/utils.rs

use gtk4::prelude::*;

/// Busca un widget por su nombre dentro de un contenedor
/// 
/// # Panics
/// Panic si el widget no se encuentra o no es del tipo esperado
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