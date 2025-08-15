#![cfg(feature = "gtk")]
use chrono::Local;
use glib::value::ToValue;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Button, CellRendererText, CheckButton, ComboBoxText,
    Entry, Grid, Label, ListStore, Orientation, PolicyType, ProgressBar, ScrolledWindow,
    SearchEntry, Separator, Spinner, TreeModelFilter, TreeView, TreeViewColumn,
};
use std::cell::RefCell;
use std::rc::Rc;

use freqdash::backtest::logic::{export_summary_to_csv, get_grid_summary, GridQuery};
use freqdash::backtest::model::StrategyGridRow;
use freqdash::config::init;

// Estado compartido de la aplicación
struct AppState {
    store: ListStore,
    results: Vec<StrategyGridRow>,
    is_loading: bool,
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(init());

    let app = Application::builder()
        .application_id("com.example.freqdash.gtk4")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Freqdash GTK4 - Análisis Completo de Backtests")
        .default_width(1600)
        .default_height(800)
        .build();

    // Contenedor principal
    let main_box = gtk4::Box::new(Orientation::Horizontal, 0);

    // Panel izquierdo
    let left_panel = create_left_panel();
    left_panel.set_size_request(350, -1);
    main_box.append(&left_panel);

    // Separador
    main_box.append(&Separator::new(Orientation::Vertical));

    // Panel derecho
    let right_panel = gtk4::Box::new(Orientation::Vertical, 0);
    right_panel.set_hexpand(true);

    // Crear store y filter
    let store = create_list_store();
    let filter_model = TreeModelFilter::new(&store, None);

    // Configurar función de filtrado basada en columna visible
    filter_model.set_visible_column(30);

    // Estado compartido
    let state = Rc::new(RefCell::new(AppState {
        store: store.clone(),
        results: Vec::new(),
        is_loading: false,
    }));

    // Toolbar de resultados
    let toolbar = create_toolbar();
    right_panel.append(&toolbar);
    right_panel.append(&Separator::new(Orientation::Horizontal));

    // Vista de tabla
    let table_view = create_table_view(&filter_model);
    let scrolled = ScrolledWindow::builder()
        .child(&table_view)
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .build();
    right_panel.append(&scrolled);

    // Barra de estado
    let status_bar = create_status_bar();
    right_panel.append(&Separator::new(Orientation::Horizontal));
    right_panel.append(&status_bar);

    main_box.append(&right_panel);

    // Conectar eventos
    connect_events(&left_panel, &toolbar, &state, &table_view, &status_bar);

    window.set_child(Some(&main_box));
    window.show();
}

fn create_left_panel() -> gtk4::Box {
    let panel = gtk4::Box::new(Orientation::Vertical, 12);
    panel.set_margin_top(12);
    panel.set_margin_bottom(12);
    panel.set_margin_start(12);
    panel.set_margin_end(12);

    // Título
    let title = Label::new(Some("Parámetros de Consulta"));
    title.set_markup("<b>Parámetros de Consulta</b>");
    title.set_halign(Align::Start);
    panel.append(&title);

    // Grid para parámetros
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
    let months_spin = gtk4::SpinButton::with_range(1.0, 24.0, 1.0);
    months_spin.set_value(6.0);
    months_spin.set_widget_name("months");
    grid.attach(&months_spin, 1, 4, 1, 1);

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

    // Separador
    panel.append(&Separator::new(Orientation::Horizontal));

    // Filtros
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

    // Checkboxes
    let profit_check = CheckButton::with_label("Solo profit > 0");
    profit_check.set_widget_name("filter_profit");
    profit_check.set_margin_top(8);
    panel.append(&profit_check);

    let winrate_check = CheckButton::with_label("Win rate > 50%");
    winrate_check.set_widget_name("filter_winrate");
    panel.append(&winrate_check);

    let trades_check = CheckButton::with_label("Trades > 100");
    trades_check.set_widget_name("filter_trades");
    panel.append(&trades_check);

    let positive_pf = CheckButton::with_label("Profit Factor > 1");
    positive_pf.set_widget_name("filter_pf");
    panel.append(&positive_pf);

    let positive_expectancy = CheckButton::with_label("Expectancy > 0");
    positive_expectancy.set_widget_name("filter_expectancy");
    panel.append(&positive_expectancy);

    panel
}

fn create_list_store() -> ListStore {
    ListStore::new(&[
        String::static_type(), // 0: Estrategia
        String::static_type(), // 1: TF
        String::static_type(), // 2: ROI
        f64::static_type(),    // 3: Stop Loss
        i32::static_type(),    // 4: Max Open Trades
        bool::static_type(),   // 5: Trailing Stop
        f64::static_type(),    // 6: TS Positive
        f64::static_type(),    // 7: TS Positive Offset
        bool::static_type(),   // 8: Trigger On Order Reached
        String::static_type(), // 9: Entry Price
        String::static_type(), // 10: Exit Price
        bool::static_type(),   // 11: Depth Market
        f64::static_type(),    // 12: Total Profit
        i32::static_type(),    // 13: Total Trades
        i32::static_type(),    // 14: Total Wins
        f64::static_type(),    // 15: Win Rate
        f64::static_type(),    // 16: Win Time
        f64::static_type(),    // 17: Drawdown %
        i32::static_type(),    // 18: Rejected Signals
        i32::static_type(),    // 19: Negative Months
        f64::static_type(),    // 20: Avg Monthly Profit
        f64::static_type(),    // 21: Std Monthly Profit
        f64::static_type(),    // 22: Max Profit Month
        f64::static_type(),    // 23: Min Profit Month
        f64::static_type(),    // 24: Avg Trade Profit
        i32::static_type(),    // 25: Losses
        f64::static_type(),    // 26: Loss Rate
        f64::static_type(),    // 27: Expectancy
        f64::static_type(),    // 28: Profit Factor
        String::static_type(), // 29: Estrategia lowercase para búsqueda
        bool::static_type(),   // 30: Visible (para compatibilidad)
    ])
}

fn create_table_view(model: &TreeModelFilter) -> TreeView {
    let tree_view = TreeView::with_model(model);
    tree_view.set_enable_search(true);
    tree_view.set_search_column(0);

    // Columnas principales con su índice en el ListStore
    let columns = [
        ("Estrategia", 0, 120, false),
        ("TF", 1, 50, false),
        ("ROI", 2, 80, false),
        ("SL", 3, 60, true),
        ("MT", 4, 80, false),
        ("TS", 5, 40, false),
        ("TProf %", 12, 100, true),
        ("Trades", 13, 60, false),
        ("Wins", 14, 50, false),
        ("WRate %", 15, 80, true),
        ("Win Time", 16, 70, true),
        ("Drawdown %", 17, 90, true),
        ("Rejected", 18, 70, false),
        ("Neg Mont", 19, 80, false),
        ("Avg Mont", 20, 90, true),
        ("Expectancy", 27, 80, true),
        ("Prof Factor", 28, 90, true),
    ];

    for (title, column_id, width, is_numeric) in columns {
        let col = TreeViewColumn::new();
        col.set_title(title);
        col.set_resizable(true);
        col.set_sort_column_id(column_id);
        col.set_min_width(width);

        let cell = CellRendererText::new();
        col.pack_start(&cell, true);

        // Formateo según columna
        match column_id {
            3 => {
                // Stop Loss
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, 3).get::<f64>() {
                        cell.set_property("text", &format!("{:.1}", value));
                    }
                });
            }
            5 => {
                // Trailing Stop (bool)
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, 5).get::<bool>() {
                        cell.set_property("text", if value { "✓" } else { "" });
                    }
                });
            }
            12 => {
                // Total Profit
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, 12).get::<f64>() {
                        let text = format!("{:.2}", value);
                        let color = if value > 0.0 { "green" } else { "red" };
                        cell.set_property("text", &text);
                        cell.set_property("foreground", color);
                    }
                });
            }
            15 => {
                // Win Rate
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, 15).get::<f64>() {
                        let text = format!("{:.1}", value * 100.0);
                        let color = if value > 0.5 { "green" } else { "orange" };
                        cell.set_property("text", &text);
                        cell.set_property("foreground", color);
                    }
                });
            }
            16 => { // Win Time (en segundos, convertir a días hh:mm)
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(seconds) = model.get_value(iter, 16).get::<f64>() {
                        // Convertir segundos a días, horas y minutos
                        let total_seconds = seconds as i64;
                        let days = total_seconds / 86400; // 86400 segundos en un día
                        let remaining_seconds = total_seconds % 86400;
                        let hours = remaining_seconds / 3600;
                        let minutes = (remaining_seconds % 3600) / 60;
                        
                        // Siempre mostrar formato con días para consistencia
                        let text = format!("{}d {:02}:{:02}", days, hours, minutes);
                        
                        cell.set_property("text", &text);
                        //cell.set_property("xalign", 0.0f32); // Alinear a la izquierda
                        cell.set_property("xalign", 1.0f32); // Alinear a la derecha
                    }
                });
            }
            17 => {
                // Drawdown
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, 17).get::<f64>() {
                        cell.set_property("text", &format!("{:.2}", value));
                        cell.set_property("foreground", "red");
                    }
                });
            }
            20 | 27 => {
                // Avg Monthly, Expectancy
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, column_id).get::<f64>() {
                        let text = format!("{:.2}", value);
                        let color = if value > 0.0 { "green" } else { "red" };
                        cell.set_property("text", &text);
                        cell.set_property("foreground", color);
                    }
                });
            }
            28 => {
                // Profit Factor
                col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                    if let Ok(value) = model.get_value(iter, 28).get::<f64>() {
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
            _ => {
                if is_numeric {
                    col.set_cell_data_func(&cell, move |_col, cell, model, iter| {
                        if column_id == 4
                            || column_id == 13
                            || column_id == 14
                            || column_id == 18
                            || column_id == 19
                        {
                            // Enteros
                            if let Ok(value) = model.get_value(iter, column_id).get::<i32>() {
                                cell.set_property("text", &value.to_string());
                            }
                        } else {
                            // Flotantes
                            if let Ok(value) = model.get_value(iter, column_id).get::<f64>() {
                                cell.set_property("text", &format!("{:.2}", value));
                            }
                        }
                    });
                } else {
                    col.add_attribute(&cell, "text", column_id);
                }
            }
        }

        tree_view.append_column(&col);
    }

    // Habilitar selección múltiple
    tree_view
        .selection()
        .set_mode(gtk4::SelectionMode::Multiple);

    tree_view
}

fn create_toolbar() -> gtk4::Box {
    let toolbar = gtk4::Box::new(Orientation::Horizontal, 8);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);

    let results_label = Label::new(Some("Resultados: 0"));
    results_label.set_widget_name("results_count");
    toolbar.append(&results_label);

    // Spacer
    let spacer = gtk4::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    let export_button = Button::with_label("Exportar Selección");
    export_button.set_widget_name("export_selection");
    export_button.set_sensitive(false);
    toolbar.append(&export_button);

    let export_all_button = Button::with_label("Exportar Todo");
    export_all_button.set_widget_name("export_all");
    export_all_button.set_sensitive(false);
    toolbar.append(&export_all_button);

    let clear_button = Button::with_label("Limpiar");
    clear_button.set_widget_name("clear");
    toolbar.append(&clear_button);

    toolbar
}

fn create_status_bar() -> gtk4::Box {
    let status_bar = gtk4::Box::new(Orientation::Horizontal, 8);
    status_bar.set_margin_top(8);
    status_bar.set_margin_bottom(8);
    status_bar.set_margin_start(8);
    status_bar.set_margin_end(8);

    let status_label = Label::new(Some("Listo"));
    status_label.set_widget_name("status");
    status_bar.append(&status_label);

    let spinner = Spinner::new();
    spinner.set_widget_name("spinner");
    spinner.set_visible(false);
    status_bar.append(&spinner);

    status_bar
}

fn connect_events(
    left_panel: &gtk4::Box,
    toolbar: &gtk4::Box,
    state: &Rc<RefCell<AppState>>,
    table_view: &TreeView,
    status_bar: &gtk4::Box,
) {
    // Botón ejecutar
    let execute_button: Button = find_widget(left_panel, "execute");
    let progress_bar: ProgressBar = find_widget(left_panel, "progress");
    let status_label: Label = find_widget(status_bar, "status");
    let spinner: Spinner = find_widget(status_bar, "spinner");

    let state_clone = state.clone();
    let left_panel_clone = left_panel.clone();
    let toolbar_clone = toolbar.clone();
    let status_bar_clone = status_bar.clone();

    execute_button.connect_clicked(move |button| {
        let mut state = state_clone.borrow_mut();
        if state.is_loading {
            return;
        }

        state.is_loading = true;
        state.store.clear();
        state.results.clear();

        // UI feedback
        button.set_sensitive(false);
        progress_bar.set_visible(true);
        progress_bar.pulse();
        status_label.set_text("Ejecutando consulta...");
        spinner.set_visible(true);
        spinner.start();

        // Obtener parámetros
        let query = get_query_params(&left_panel_clone);

        // Timer para progress
        let progress_clone = progress_bar.clone();
        let timeout_id =
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                progress_clone.pulse();
                glib::ControlFlow::Continue
            });

        let state_clone2 = state_clone.clone();
        let button_clone = button.clone();
        let progress_clone = progress_bar.clone();
        let toolbar_clone2 = toolbar_clone.clone();
        let status_bar_clone2 = status_bar_clone.clone();

        glib::MainContext::default().spawn_local(async move {
            match get_grid_summary(&query).await {
                Ok(mut rows) => {
                    // Ordenar por profit
                    rows.sort_by(|a, b| {
                        b.total_profit
                            .partial_cmp(&a.total_profit)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let mut state = state_clone2.borrow_mut();
                    state.results = rows.clone();

                    // Poblar store con todos los campos
                    for r in &rows {
                        state.store.insert_with_values(
                            None,
                            &[
                                (0, &r.strategy.to_value()),
                                (1, &r.timeframe.to_value()),
                                (2, &r.minimal_roi.to_value()),
                                (3, &r.stoploss.parse::<f64>().unwrap_or(-99.0).to_value()),
                                (4, &r.max_open_trades.to_value()),
                                (5, &r.trailing_stop.to_value()),
                                (6, &r.trailing_stop_positive.unwrap_or(0.0).to_value()),
                                (
                                    7,
                                    &r.trailing_stop_positive_offset.unwrap_or(0.0).to_value(),
                                ),
                                (8, &r.trailing_only_offset_is_reached.to_value()),
                                (9, &r.entry_price.to_value()),
                                (10, &r.exit_price.to_value()),
                                (11, &r.check_depth_of_market_enable.to_value()),
                                (12, &r.total_profit.to_value()),
                                (13, &r.total_trades.to_value()),
                                (14, &r.wins.to_value()),
                                (15, &r.win_rate.to_value()),
                                (16, &r.win_time.to_value()),
                                (17, &r.drawdown_perc.to_value()),
                                (18, &(r.rejected_signals as i32).to_value()),
                                (19, &(r.neg_months as i32).to_value()),
                                (20, &r.avg_monthly_profit.to_value()),
                                (21, &r.std_monthly_profit.to_value()),
                                (22, &r.max_profit_month.to_value()),
                                (23, &r.min_profit_month.to_value()),
                                (24, &r.avg_trade_profit.to_value()),
                                (25, &r.losses.to_value()),
                                (26, &r.loss_rate.to_value()),
                                (27, &r.expectancy.to_value()),
                                (28, &r.profit_factor.to_value()),
                                (29, &r.strategy.to_lowercase().to_value()),
                                (30, &true.to_value()),
                            ],
                        );
                    }

                    // Actualizar UI
                    update_results_count(&toolbar_clone2, rows.len());
                    update_status(
                        &status_bar_clone2,
                        &format!("{} estrategias encontradas", rows.len()),
                    );
                    enable_export_buttons(&toolbar_clone2, true);

                    state.is_loading = false;
                }
                Err(e) => {
                    update_status(&status_bar_clone2, &format!("Error: {}", e));
                    state_clone2.borrow_mut().is_loading = false;
                }
            }

            // Restaurar UI
            button_clone.set_sensitive(true);
            progress_clone.set_visible(false);
            timeout_id.remove();
            let spinner: Spinner = find_widget(&status_bar_clone2, "spinner");
            spinner.stop();
            spinner.set_visible(false);
        });
    });

    // Filtros
    setup_filters(left_panel, state);

    // Exportación
    setup_export(toolbar, state, table_view);
}

fn setup_filters(panel: &gtk4::Box, state: &Rc<RefCell<AppState>>) {
    let search: SearchEntry = find_widget(panel, "search");
    let profit_check: CheckButton = find_widget(panel, "filter_profit");
    let winrate_check: CheckButton = find_widget(panel, "filter_winrate");
    let trades_check: CheckButton = find_widget(panel, "filter_trades");
    let pf_check: CheckButton = find_widget(panel, "filter_pf");
    let expectancy_check: CheckButton = find_widget(panel, "filter_expectancy");

    let state_clone = state.clone();

    // Clone the widgets before moving them into the closure
    let search_clone = search.clone();
    let profit_check_clone = profit_check.clone();
    let winrate_check_clone = winrate_check.clone();
    let trades_check_clone = trades_check.clone();
    let pf_check_clone = pf_check.clone();
    let expectancy_check_clone = expectancy_check.clone();

    let apply_filters = move || {
        let search_text = search_clone.text().to_string().to_lowercase();
        let filter_profit = profit_check_clone.is_active();
        let filter_winrate = winrate_check_clone.is_active();
        let filter_trades = trades_check_clone.is_active();
        let filter_pf = pf_check_clone.is_active();
        let filter_expectancy = expectancy_check_clone.is_active();

        let state = state_clone.borrow();
        let store = &state.store;

        if let Some(iter) = store.iter_first() {
            loop {
                let mut visible = true;

                // Búsqueda
                if !search_text.is_empty() {
                    if let Ok(strategy) = store.get_value(&iter, 29).get::<String>() {
                        if !strategy.contains(&search_text) {
                            visible = false;
                        }
                    }
                }

                // Filtros
                if visible && filter_profit {
                    if let Ok(profit) = store.get_value(&iter, 12).get::<f64>() {
                        if profit <= 0.0 {
                            visible = false;
                        }
                    }
                }

                if visible && filter_winrate {
                    if let Ok(winrate) = store.get_value(&iter, 15).get::<f64>() {
                        if winrate <= 0.5 {
                            visible = false;
                        }
                    }
                }

                if visible && filter_trades {
                    if let Ok(trades) = store.get_value(&iter, 13).get::<i32>() {
                        if trades <= 100 {
                            visible = false;
                        }
                    }
                }

                if visible && filter_pf {
                    if let Ok(pf) = store.get_value(&iter, 28).get::<f64>() {
                        if pf <= 1.0 {
                            visible = false;
                        }
                    }
                }

                if visible && filter_expectancy {
                    if let Ok(expectancy) = store.get_value(&iter, 27).get::<f64>() {
                        if expectancy <= 0.0 {
                            visible = false;
                        }
                    }
                }

                store.set_value(&iter, 30, &visible.to_value());

                if !store.iter_next(&iter) {
                    break;
                }
            }
        }
    };

    let filters_clone = Rc::new(apply_filters);

    search.connect_search_changed({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    profit_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    winrate_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    trades_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    pf_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });

    expectancy_check.connect_toggled({
        let filters = filters_clone.clone();
        move |_| filters()
    });
}

fn setup_filter_function(state: &Rc<RefCell<AppState>>) {
    // Esta función ya no es necesaria
}

fn setup_export(toolbar: &gtk4::Box, state: &Rc<RefCell<AppState>>, table_view: &TreeView) {
    let export_selection: Button = find_widget(toolbar, "export_selection");
    let export_all: Button = find_widget(toolbar, "export_all");
    let clear: Button = find_widget(toolbar, "clear");

    // Exportar selección
    let state_clone = state.clone();
    let tree_view_clone = table_view.clone();
    export_selection.connect_clicked(move |_| {
        let selection = tree_view_clone.selection();
        let (paths, model) = selection.selected_rows();

        if !paths.is_empty() {
            let state = state_clone.borrow();
            let mut selected = Vec::new();

            for path in paths {
                if let Some(iter) = model.iter(&path) {
                    if let Ok(strategy) = model.get_value(&iter, 0).get::<String>() {
                        if let Some(result) = state.results.iter().find(|r| r.strategy == strategy)
                        {
                            selected.push(result.clone());
                        }
                    }
                }
            }

            if !selected.is_empty() {
                let filename = format!("selection_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
                export_summary_to_csv(&selected, &filename);
                println!("Exportado {} estrategias a {}", selected.len(), filename);
            }
        }
    });

    // Exportar todo
    let state_clone = state.clone();
    export_all.connect_clicked(move |_| {
        let state = state_clone.borrow();
        if !state.results.is_empty() {
            let filename = format!("all_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
            export_summary_to_csv(&state.results, &filename);
            println!(
                "Exportado {} estrategias a {}",
                state.results.len(),
                filename
            );
        }
    });

    // Limpiar
    let state_clone = state.clone();
    let toolbar_clone = toolbar.clone();
    clear.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.store.clear();
        state.results.clear();
        update_results_count(&toolbar_clone, 0);
        enable_export_buttons(&toolbar_clone, false);
    });
}

// Funciones auxiliares
fn find_widget<T: IsA<gtk4::Widget>>(parent: &impl IsA<gtk4::Widget>, name: &str) -> T {
    let mut queue = vec![parent.clone().upcast::<gtk4::Widget>()];

    while let Some(widget) = queue.pop() {
        if widget.widget_name() == name {
            return widget
                .downcast::<T>()
                .expect(&format!("Widget {} not found or wrong type", name));
        }

        let mut child = widget.first_child();
        while let Some(c) = child {
            queue.push(c.clone());
            child = c.next_sibling();
        }
    }

    panic!("Widget {} not found", name);
}

fn get_query_params(panel: &gtk4::Box) -> GridQuery {
    let exchange: ComboBoxText = find_widget(panel, "exchange");
    let currency: ComboBoxText = find_widget(panel, "currency");
    let pairlist: Entry = find_widget(panel, "pairlist");
    let start_date: Entry = find_widget(panel, "start_date");
    let months: gtk4::SpinButton = find_widget(panel, "months");

    GridQuery {
        exchange: exchange
            .active_text()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "BINANCE".to_string()),
        currency: currency
            .active_text()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "USDT".to_string()),
        pairlist: pairlist.text().to_string(),
        start_date: start_date.text().to_string(),
        months: months.value() as usize,
    }
}

fn update_status(status_bar: &gtk4::Box, message: &str) {
    let label: Label = find_widget(status_bar, "status");
    label.set_text(message);
}

fn update_results_count(toolbar: &gtk4::Box, count: usize) {
    let label: Label = find_widget(toolbar, "results_count");
    label.set_text(&format!("Resultados: {}", count));
}

fn enable_export_buttons(toolbar: &gtk4::Box, enable: bool) {
    let export_selection: Button = find_widget(toolbar, "export_selection");
    let export_all: Button = find_widget(toolbar, "export_all");

    export_selection.set_sensitive(enable);
    export_all.set_sensitive(enable);
}
