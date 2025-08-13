// src/gui_adw/ui/left_panel.rs

use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, Entry, Label, Orientation, ProgressBar, SearchEntry, Separator, SpinButton,
    StringList,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, Clamp, ComboRow, PreferencesGroup};

/// Crea el panel izquierdo con controles de consulta y filtros usando componentes Adwaita
pub fn create() -> Box {
    let panel = Box::new(Orientation::Vertical, 12);
    panel.add_css_class("background");
    panel.set_margin_top(12);
    panel.set_margin_bottom(12);
    panel.set_margin_start(12);
    panel.set_margin_end(12);

    // Usar Clamp de Adwaita para mejor responsividad
    let clamp = Clamp::new();
    clamp.set_maximum_size(350);
    clamp.set_tightening_threshold(300);

    let content_box = Box::new(Orientation::Vertical, 12);

    // Sección de parámetros usando PreferencesGroup de Adwaita
    let query_group = build_query_section();
    content_box.append(&query_group);

    // Botón ejecutar con estilo Adwaita
    let execute_button = Button::with_label("Ejecutar Consulta");
    execute_button.add_css_class("suggested-action");
    execute_button.add_css_class("pill");
    execute_button.set_widget_name("execute");
    execute_button.set_margin_top(12);
    execute_button.set_margin_bottom(12);
    content_box.append(&execute_button);

    // Progress bar
    let progress_bar = ProgressBar::new();
    progress_bar.set_widget_name("progress");
    progress_bar.set_visible(false);
    progress_bar.add_css_class("osd");
    content_box.append(&progress_bar);

    // Separador
    content_box.append(&Separator::new(Orientation::Horizontal));

    // Sección de filtros usando PreferencesGroup
    let filter_group = build_filter_section();
    content_box.append(&filter_group);

    clamp.set_child(Some(&content_box));
    panel.append(&clamp);

    panel
}

/// Construye la sección de parámetros con componentes Adwaita
fn build_query_section() -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Parámetros de Consulta");
    group.set_description(Some(
        "Configure los parámetros para la consulta de backtest",
    ));

    // Exchange ComboRow
    let exchange_row = ComboRow::new();
    exchange_row.set_title("Exchange");
    exchange_row.set_subtitle("Seleccione el exchange");
    let exchange_model = gtk4::StringList::new(&["BINANCE", "KRAKEN"]);
    exchange_row.set_model(Some(&exchange_model));
    exchange_row.set_selected(0);
    exchange_row.set_widget_name("exchange");
    group.add(&exchange_row);

    // Currency ComboRow
    let currency_row = ComboRow::new();
    currency_row.set_title("Moneda");
    currency_row.set_subtitle("Moneda base para el trading");
    let currency_model = gtk4::StringList::new(&["USDT", "BTC"]);
    currency_row.set_model(Some(&currency_model));
    currency_row.set_selected(0);
    currency_row.set_widget_name("currency");
    group.add(&currency_row);

    /*  // Pairlist ActionRow con Entry
    let pairlist_row = ActionRow::new();
    pairlist_row.set_title("Pares");
    pairlist_row.set_subtitle("Lista de pares separados por comas");
    let pairlist_entry = Entry::builder()
        .placeholder_text("BTC,ETH,BNB")
        .text("BTC")
        .valign(Align::Center)
        .build();
    pairlist_entry.set_widget_name("pairlist");
    pairlist_row.add_suffix(&pairlist_entry);
    pairlist_row.set_activatable_widget(Some(&pairlist_entry));
    group.add(&pairlist_row); */

    let pairlist_row = ComboRow::builder()
        .title("Pares")
        .subtitle("Lista de pares disponibles")
        .name("pairlist_combo")
        .model(&StringList::new(&[]))
        .build();
    group.add(&pairlist_row);

    // Fecha ActionRow
    let date_row = ActionRow::new();
    date_row.set_title("Fecha de Inicio");
    date_row.set_subtitle("Formato YYYY-MM-DD");
    let start_entry = Entry::builder()
        .placeholder_text("YYYY-MM-DD")
        .text("2024-01-01")
        .valign(Align::Center)
        .build();
    start_entry.set_widget_name("start_date");
    date_row.add_suffix(&start_entry);
    date_row.set_activatable_widget(Some(&start_entry));
    group.add(&date_row);

    // Meses ActionRow con SpinButton
    let months_row = ActionRow::new();
    months_row.set_title("Número de Meses");
    months_row.set_subtitle("Período de análisis");
    let months_spin = SpinButton::with_range(1.0, 24.0, 1.0);
    months_spin.set_value(6.0);
    months_spin.set_widget_name("months");
    months_spin.set_valign(Align::Center);
    months_row.add_suffix(&months_spin);
    months_row.set_activatable_widget(Some(&months_spin));
    group.add(&months_row);

    group
}

/// Construye la sección de filtros con componentes Adwaita
fn build_filter_section() -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Filtros");
    group.set_description(Some("Filtros para refinar los resultados"));

    // Búsqueda con ActionRow
    let search_row = ActionRow::new();
    search_row.set_title("Buscar");
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Buscar estrategia..."));
    search_entry.set_widget_name("search");
    search_entry.set_valign(Align::Center);
    search_entry.set_hexpand(true);
    search_row.add_suffix(&search_entry);
    search_row.set_activatable_widget(Some(&search_entry));
    group.add(&search_row);

    // Switches usando ActionRow de Adwaita
    let filters = [
        (
            "filter_profit",
            "Solo Profit Positivo",
            "Mostrar solo estrategias con profit > 0",
        ),
        (
            "filter_winrate",
            "Win Rate > 50%",
            "Filtrar por tasa de acierto superior al 50%",
        ),
        (
            "filter_trades",
            "Trades > 100",
            "Mostrar solo estrategias con más de 100 trades",
        ),
        (
            "filter_pf",
            "Profit Factor > 1",
            "Filtrar por profit factor positivo",
        ),
        (
            "filter_expectancy",
            "Expectancy > 0",
            "Mostrar solo estrategias con expectancy positiva",
        ),
    ];

    for (name, title, subtitle) in filters {
        let row = ActionRow::new();
        row.set_title(title);
        row.set_subtitle(subtitle);

        let switch = gtk4::Switch::new();
        switch.set_widget_name(name);
        switch.set_valign(Align::Center);
        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));

        group.add(&row);
    }

    group
}
