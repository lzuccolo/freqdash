// src/gui/ui/table_view.rs

use gtk4::prelude::*;
use gtk4::{
    CellRendererText, ListStore, SelectionMode, TreeModelFilter,
    TreeView, TreeViewColumn,
};
use glib::Type;

/// Crea la vista de tabla con el modelo filtrado
pub fn create(filter_model: &TreeModelFilter) -> TreeView {
    let tree_view = TreeView::with_model(filter_model);
    tree_view.set_enable_search(true);
    tree_view.set_search_column(0);

    setup_columns(&tree_view);
    
    // Habilitar selección múltiple
    tree_view
        .selection()
        .set_mode(SelectionMode::Multiple);

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

/// Configura todas las columnas de la tabla
fn setup_columns(tree_view: &TreeView) {
    // Definición de columnas: (título, índice, ancho, es_numérico)
    let columns = [
        ("Estrategia", 0, 120, false),
        ("TF", 1, 50, false),
        ("Min ROI", 2, 80, false),
        ("SL", 3, 60, true),
        ("Max Trades", 4, 80, false),
        ("TS", 5, 40, false),
        ("Total Profit (%)", 12, 100, true),
        ("Trades", 13, 60, false),
        ("Wins", 14, 50, false),
        ("Win Rate (%)", 15, 80, true),
        ("Win Time", 16, 90, true),
        ("Drawdown (%)", 17, 90, true),
        ("Rejected", 18, 70, false),
        ("Neg Months", 19, 80, false),
        ("Avg Monthly", 20, 90, true),
        ("Expectancy", 27, 80, true),
        ("Profit Factor", 28, 90, true),
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

/// Configura el renderizado de cada celda según su tipo
fn setup_cell_renderer(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    match column_id {
        3 => format_stop_loss(col, cell),
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
            cell.set_property("text", &format!("{:.1}", value));
        }
    });
}

fn format_trailing_stop(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 5).get::<bool>() {
            cell.set_property("text", if value { "✓" } else { "" });
        }
    });
}

fn format_total_profit(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 12).get::<f64>() {
            let text = format!("{:.2}", value);
            let color = if value > 0.0 { "green" } else { "red" };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
        }
    });
}

fn format_win_rate(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 15).get::<f64>() {
            let text = format!("{:.1}", value * 100.0);
            let color = if value > 0.5 { "green" } else { "orange" };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
        }
    });
}

fn format_win_time(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(seconds) = model.get_value(&iter, 16).get::<f64>() {
            // Convertir segundos a días, horas y minutos
            let total_seconds = seconds as i64;
            let days = total_seconds / 86400;
            let remaining_seconds = total_seconds % 86400;
            let hours = remaining_seconds / 3600;
            let minutes = (remaining_seconds % 3600) / 60;
            
            let text = format!("{}d {:02}:{:02}", days, hours, minutes);
            
            cell.set_property("text", &text);
            cell.set_property("xalign", 1.0f32); // Alinear a la derecha
        }
    });
}

fn format_drawdown(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 17).get::<f64>() {
            cell.set_property("text", &format!("{:.2}", value));
            cell.set_property("foreground", "red");
        }
    });
}

fn format_colored_numeric(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, column_id).get::<f64>() {
            let text = format!("{:.2}", value);
            let color = if value > 0.0 { "green" } else { "red" };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
        }
    });
}

fn format_profit_factor(col: &TreeViewColumn, cell: &CellRendererText) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, 28).get::<f64>() {
            let text = format!("{:.2}", value);
            let color = if value > 1.0 { 
                "green" 
            } else if value > 0.8 { 
                "orange" 
            } else { 
                "red" 
            };
            cell.set_property("text", &text);
            cell.set_property("foreground", color);
        }
    });
}

fn format_integer(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, column_id).get::<i32>() {
            cell.set_property("text", &value.to_string());
        }
    });
}

fn format_text(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.add_attribute(cell, "text", column_id);
}

fn format_float(col: &TreeViewColumn, cell: &CellRendererText, column_id: i32) {
    col.set_cell_data_func(cell, move |_col, cell, model, iter| {
        if let Ok(value) = model.get_value(&iter, column_id).get::<f64>() {
            cell.set_property("text", &format!("{:.2}", value));
        }
    });
}