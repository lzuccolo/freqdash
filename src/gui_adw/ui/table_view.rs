// src/gui_adw/ui/table_view.rs

use glib::Type;
use gtk4::pango::Style;
use gtk4::prelude::*;
use gtk4::{CellRendererText, ListStore, SelectionMode, TreeModelFilter, TreeView, TreeViewColumn};
use gtk4::CenterBox;
use gtk4::Label;


/// Crea la vista de tabla con el modelo filtrado y estilo Adwaita
pub fn create(filter_model: &TreeModelFilter) -> TreeView {
    let tree_view = TreeView::with_model(filter_model);
    tree_view.set_enable_search(true);
    tree_view.set_search_column(0);
    tree_view.add_css_class("rich-list");
    // set_show_row_separators no existe en GTK4, usar CSS en su lugar

    setup_columns(&tree_view);

    // Habilitar selección múltiple
    tree_view.selection().set_mode(SelectionMode::Multiple);

    tree_view
}

/// Crea el ListStore con todas las columnas necesarias
pub fn create_list_store() -> ListStore {
    ListStore::new(&[
        Type::STRING, // 0: Estrategia
        Type::STRING, // 1: TF
        Type::STRING, // 2: ROI
        Type::F64,    // 3: Stop Loss
        Type::I32,    // 4: Max Open Trades
        Type::BOOL,   // 5: Trailing Stop
        Type::F64,    // 6: TS Positive
        Type::F64,    // 7: TS Positive Offset
        Type::BOOL,   // 8: Trigger On Order Reached
        Type::STRING, // 9: Entry Price
        Type::STRING, // 10: Exit Price
        Type::BOOL,   // 11: Depth Market
        Type::F64,    // 12: Total Profit
        Type::I32,    // 13: Total Trades
        Type::I32,    // 14: Total Wins
        Type::F64,    // 15: Win Rate
        Type::F64,    // 16: Win Time (en segundos)
        Type::F64,    // 17: Drawdown %
        Type::I32,    // 18: Rejected Signals
        Type::I32,    // 19: Negative Months
        Type::F64,    // 20: Avg Monthly Profit
        Type::F64,    // 21: Std Monthly Profit
        Type::F64,    // 22: Max Profit Month
        Type::F64,    // 23: Min Profit Month
        Type::F64,    // 24: Avg Trade Profit
        Type::I32,    // 25: Losses
        Type::F64,    // 26: Loss Rate
        Type::F64,    // 27: Expectancy
        Type::F64,    // 28: Profit Factor
        Type::STRING, // 29: Estrategia lowercase para búsqueda
        Type::BOOL,   // 30: Visible (para filtrado)
    ])
}

/// Obtiene el ListStore base desde un TreeModelFilter
pub fn get_base_store(filter_model: &TreeModelFilter) -> ListStore {
    filter_model
        .model()
        .downcast::<ListStore>()
        .expect("El modelo debe ser un ListStore")
}

/// Configura todas las columnas de la tabla con estilo mejorado
fn setup_columns(tree_view: &TreeView) {
    // Definición de columnas con iconos cuando sea apropiado
    let columns = [
        ("📊 Estrategia", 0, 140, false),
        ("⏱ TF", 1, 60, false),
        ("📈 Min ROI", 2, 90, false),
        ("🛑 SL", 3, 70, true),
        ("🔢 Max T", 4, 100, false),
        ("🎯 TS", 5, 50, false),
        ("💰 T.Profit", 12, 120, true),
        ("📊 Trades", 13, 80, false),
        ("✅ Wins", 14, 70, false),
        ("🎯 Win Rate", 15, 100, true),
        ("⏳ Win Time", 16, 100, true),
        ("📉 Drawdown", 17, 110, true),
        ("❌ Rejected", 18, 90, false),
        ("🔴 Neg Mon", 19, 100, false),
        ("📊 Avg Mon", 20, 110, true),
        ("🎲 Expect", 27, 100, true),
        ("⚖️ P.Factor", 28, 110, true),
    ];

    for (title, column_id, width, _is_numeric) in columns {
        let col = TreeViewColumn::new();
        col.set_title(title);
        col.set_resizable(true);
        col.set_sort_column_id(column_id);
        col.set_min_width(width);

        let cell = CellRendererText::new();
        col.pack_start(&cell, true);

        setup_cell_renderer(&col, &cell, column_id);
        tree_view.append_column(&col);
    }
}

/// Configura el renderizado de cada celda con estilos mejorados
fn setup_cell_renderer(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    cell.set_padding(8, 4); // Aplicar padding uniforme
    cell.set_property("xalign", 1.0_f32); //Alinear todo a la derecha por defecto
                                          //cell.set_property("xalign", 0.5_f32); // <-- Centrar
                                          //cell.set_property("xalign", 0.0_f32); // <-- Alinear a la izquierda

    match column_id {
        // 3 => format_stop_loss(col, cell),
        5 => format_trailing_stop(col, cell),
        12 => format_total_profit(col, cell),
        15 => format_win_rate(col, cell),
        16 => format_win_time(col, cell),
        17 => format_drawdown(col, cell),
        20 | 27 => format_colored_numeric(col, cell, column_id),
        28 => format_profit_factor(col, cell),
        4 | 13 | 14 | 18 | 19 => format_integer(col, cell, column_id),
        0 | 1 | 2 | 9 | 10 => format_text(col, cell, column_id),
        _ => format_float(col, cell, column_id),
    }
}

// Funciones de formateo específicas para cada tipo de columna

fn format_stop_loss(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 3).get::<f64>() {
            cell.set_property("text", &format!("{:.1}%", value));
            cell.set_property("weight", 600);
        }
    });
}

fn format_trailing_stop(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 5).get::<bool>() {
            cell.set_property("text", if value { "✅" } else { "❌" });
        }
    });
}

fn format_total_profit(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 12).get::<f64>() {
            let text = format!("{:.1}%", value * 100.0);
            let (color, weight) = if value > 10.0 {
                ("#2ec27e", 700) // Verde brillante para muy bueno
            } else if value > 0.0 {
                ("#57e389", 600) // Verde normal
            } else if value > -5.0 {
                ("#f6d32d", 500) // Amarillo para neutro
            } else {
                ("#e01b24", 600) // Rojo para malo
            };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
            cell.set_property("weight", weight);
        }
    });
}

fn format_win_rate(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 15).get::<f64>() {
            let percentage = value * 100.0;
            let text = format!("{:.1}%", percentage);
            let (color, weight) = if percentage > 60.0 {
                ("#2ec27e", 700)
            } else if percentage > 50.0 {
                ("#57e389", 600)
            } else if percentage > 40.0 {
                ("#f6d32d", 500)
            } else {
                ("#ff7800", 600)
            };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
            cell.set_property("weight", weight);
        }
    });
}

fn format_win_time(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(seconds) = model.get_value(&iter, 16).get::<f64>() {
            let total_seconds = seconds as i64;
            let days = total_seconds / 86400;
            let remaining_seconds = total_seconds % 86400;
            let hours = remaining_seconds / 3600;
            let minutes = (remaining_seconds % 3600) / 60;

            let text = if days > 0 {
                format!("{}d {:02}h:{:02}m", days, hours, minutes)
            } else {
                format!("{:02}h:{:02}m", hours, minutes)
            };

            cell.set_property("text", &text);
            cell.set_property("font", "Monospace");
            cell.set_property("xalign", 1.0f32);
        }
    });
}

fn format_drawdown(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 17).get::<f64>() {
            let percentage = value * 100.0;
            let text = format!("-{:.1}%", percentage.abs());
            let (color, weight) = if percentage < 5.0 {
                ("#57e389", 500)
            } else if value < 10.0 {
                ("#f6d32d", 600)
            } else if value < 20.0 {
                ("#ff7800", 600)
            } else {
                ("#e01b24", 700)
            };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
            cell.set_property("weight", weight);
        }
    });
}

fn format_colored_numeric(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, column_id).get::<f64>() {
            let text = format!("{:.2}", value);
            let (color, weight) = if value > 0.0 {
                ("#2ec27e", 600)
            } else if value == 0.0 {
                ("#77767b", 500)
            } else {
                ("#e01b24", 600)
            };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
            cell.set_property("weight", weight);
        }
    });
}

fn format_profit_factor(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 28).get::<f64>() {
            let text = format!("{:.2}", value);
            let (color, weight, style) = if value > 2.0 {
                ("#2ec27e", 700, "italic")
            } else if value > 1.5 {
                ("#57e389", 600, "normal")
            } else if value > 1.0 {
                ("#f6d32d", 500, "normal")
            } else if value > 0.8 {
                ("#ff7800", 600, "normal")
            } else {
                ("#e01b24", 700, "normal")
            };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
            cell.set_property("weight", weight);
            cell.set_property("style", &Style::Normal);
        }
    });
}

fn format_integer(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, column_id).get::<i32>() {
            let text = value.to_string();
            cell.set_property("text", &text);

            // Aplicar colores especiales para ciertas columnas
            if column_id == 19 && value > 0 {
                // Meses negativos
                cell.set_property("foreground", "#ff7800");
                cell.set_property("weight", 600);
            }
        }
    });
}

fn format_text(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.add_attribute(cell, "text", column_id);

    // Aplicar estilos especiales para ciertas columnas
    if column_id == 0 {
        // Estrategia
        cell.set_property("weight", 600);
        cell.set_property("xalign", 0.0_f32); // <-- Alinear a la izquierda
    } else if column_id == 1 {
        // Timeframe
        cell.set_property("font", "Monospace");
    }
}

fn format_float(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, column_id).get::<f64>() {
            cell.set_property("text", &format!("{:.2}", value));
        }
    });
}
