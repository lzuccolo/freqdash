use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, ComboBoxText, Entry, Grid, Label,
    Orientation, ProgressBar, SearchEntry, Separator, SpinButton,
};

/// Crea el panel izquierdo con controles de consulta y filtros
pub fn create() -> Box {
    let panel = Box::new(Orientation::Vertical, 12);
    panel.set_margin_top(12);
    panel.set_margin_bottom(12);
    panel.set_margin_start(12);
    panel.set_margin_end(12);

    // Sección de parámetros de consulta
    build_query_section(&panel);
    
    // Separador
    panel.append(&Separator::new(Orientation::Horizontal));
    
    // Sección de filtros
    build_filter_section(&panel);

    panel
}

/// Construye la sección de parámetros de consulta
fn build_query_section(panel: &Box) {
    // Título
    let title = Label::new(Some("Parámetros de Consulta"));
    title.set_markup("<b>Parámetros de Consulta</b>");
    title.set_halign(Align::Start);
    panel.append(&title);

    // Grid con controles
    let grid = create_query_grid();
    panel.append(&grid);

    // Botón ejecutar
    let execute_button = Button::with_label("Ejecutar Consulta");
    execute_button.add_css_class("suggested-action");
    execute_button.set_widget_name("execute");
    execute_button.set_margin_top(20);
    panel.append(&execute_button);

    // Progress bar
    let progress_bar = ProgressBar::new();
    progress_bar.set_widget_name("progress");
    progress_bar.set_visible(false);
    panel.append(&progress_bar);
}

/// Crea el grid con los controles de consulta
fn create_query_grid() -> Grid {
    let grid = Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    grid.set_margin_top(12);

    // Exchange
    grid.attach(&Label::new(Some("Exchange:")), 0, 0, 1, 1);
    let exchange_combo = ComboBoxText::new();
    exchange_combo.append_text("BINANCE");
    exchange_combo.append_text("KRAKEN");
    exchange_combo.set_active(Some(0));
    exchange_combo.set_widget_name("exchange");
    grid.attach(&exchange_combo, 1, 0, 1, 1);

    // Currency
    grid.attach(&Label::new(Some("Currency:")), 0, 1, 1, 1);
    let currency_combo = ComboBoxText::new();
    currency_combo.append_text("USDT");
    currency_combo.append_text("BTC");
    currency_combo.set_active(Some(0));
    currency_combo.set_widget_name("currency");
    grid.attach(&currency_combo, 1, 1, 1, 1);

    // Pairlist
    grid.attach(&Label::new(Some("Pairlist:")), 0, 2, 1, 1);
    let pairlist_entry = Entry::builder()
        .placeholder_text("BTC,ETH,BNB")
        .text("BTC")
        .build();
    pairlist_entry.set_widget_name("pairlist");
    grid.attach(&pairlist_entry, 1, 2, 1, 1);

    // Fecha
    grid.attach(&Label::new(Some("Fecha Inicio:")), 0, 3, 1, 1);
    let start_entry = Entry::builder()
        .placeholder_text("YYYY-MM-DD")
        .text("2024-01-01")
        .build();
    start_entry.set_widget_name("start_date");
    grid.attach(&start_entry, 1, 3, 1, 1);

    // Meses
    grid.attach(&Label::new(Some("Meses:")), 0, 4, 1, 1);
    let months_spin = SpinButton::with_range(1.0, 24.0, 1.0);
    months_spin.set_value(6.0);
    months_spin.set_widget_name("months");
    grid.attach(&months_spin, 1, 4, 1, 1);

    grid
}

/// Construye la sección de filtros
fn build_filter_section(panel: &Box) {
    // Título
    let filter_title = Label::new(Some("Filtros"));
    filter_title.set_markup("<b>Filtros</b>");
    filter_title.set_halign(Align::Start);
    filter_title.set_margin_top(12);
    panel.append(&filter_title);

    // Búsqueda
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Buscar estrategia..."));
    search_entry.set_widget_name("search");
    search_entry.set_margin_top(8);
    panel.append(&search_entry);

    // Checkboxes de filtros
    let filters = [
        ("filter_profit", "Solo profit > 0", true),
        ("filter_winrate", "Win rate > 50%", false),
        ("filter_trades", "Trades > 100", false),
        ("filter_pf", "Profit Factor > 1", false),
        ("filter_expectancy", "Expectancy > 0", false),
    ];

    for (name, label, add_margin) in filters {
        let check = CheckButton::with_label(label);
        check.set_widget_name(name);
        if add_margin {
            check.set_margin_top(8);
        }
        panel.append(&check);
    }
}