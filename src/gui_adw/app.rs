// src/gui_adw/app.rs
use adw::{NavigationPage, NavigationSplitView};
use gtk4::prelude::*;
use gtk4::Button;
use libadwaita as adw;
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use tokio::runtime::Runtime;

use crate::gui_adw::state::AppState;
use crate::gui_adw::{events, ui, utils};
use crate::{config, db};

use crate::backtest::logic::get_grid_summary;
use crate::backtest::model::{GridQuery, StrategyGridRow};
use crate::gui_adw::events::query::{self, BATCH_SIZE};

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get().expect("Runtime not initialized")
}

#[derive(Debug, Clone)]
pub enum DatabaseCommand {
    RunBacktest(GridQuery),
}

#[derive(Debug)]
pub enum DatabaseResult {
    Backtest(Result<Vec<StrategyGridRow>, String>),
}

pub fn run() {
    config::init_config();
    let rt = Runtime::new().unwrap();
    TOKIO_RUNTIME.set(rt).unwrap();
    db::init_db_pool();

    let (command_tx, command_rx) = mpsc::channel::<DatabaseCommand>();
    let (result_tx, result_rx) = mpsc::channel::<DatabaseResult>();

    std::thread::spawn(move || {
        database_worker(command_rx, result_tx);
    });

    let app = adw::Application::new(Some("com.example.freqdash"), Default::default());
    let result_rx = Rc::new(RefCell::new(result_rx));

    app.connect_activate(move |app| {
        build_ui(app, command_tx.clone(), result_rx.clone());
    });
    app.run();
}

fn build_ui(
    app: &adw::Application,
    command_tx: mpsc::Sender<DatabaseCommand>,
    result_rx: Rc<RefCell<mpsc::Receiver<DatabaseResult>>>,
) {
    let header_bar = ui::toolbar::create();
    let left_panel = ui::left_panel::create();
    left_panel.set_width_request(360);
    let (right_panel, tree_view, filter_model) = ui::right_panel::create();

    let split_view = NavigationSplitView::new();
    let content_page = NavigationPage::new(&right_panel, "Resultados");
    split_view.set_content(Some(&content_page));

    let sidebar_page = NavigationPage::new(&left_panel, "Parámetros");
    split_view.set_sidebar(Some(&sidebar_page));
    
    split_view.set_min_sidebar_width(360.0);
    split_view.set_max_sidebar_width(400.0);
    split_view.set_collapsed(false);

    let flap_toggle: Button = utils::find_widget(&header_bar, "flap_toggle");
    let split_view_clone = split_view.clone();
    flap_toggle.connect_clicked(move |_| {
        split_view_clone.set_collapsed(!split_view_clone.is_collapsed());
    });

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);
    toolbar_view.set_content(Some(&split_view));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("FreqDash")
        .default_width(1200)
        .default_height(800)
        .content(&toolbar_view)
        .build();

    let store = ui::table_view::get_base_store(&filter_model);
    let app_state = Rc::new(RefCell::new(AppState::new(store, filter_model)));

    // Clone all variables that will be moved into closures
    let command_tx_clone = command_tx.clone();
    let result_rx_clone = result_rx.clone();
    let app_state_clone = app_state.clone();
    let left_panel_clone = left_panel.clone();
    let right_panel_clone = right_panel.clone();
    let tree_view_clone = tree_view.clone();
    let header_bar_clone = header_bar.clone();

    window.connect_show(move |_| {
        // Clone variables for connect_all
        let command_tx_for_connect = command_tx_clone.clone();
        events::connect_all(
            &left_panel_clone,
            &right_panel_clone,
            &header_bar_clone,
            &tree_view_clone,
            &app_state_clone,
            command_tx_for_connect,
        );

        // Clone variables for the timer
        let result_rx_for_timer = result_rx_clone.clone();
        let app_state_for_timer = app_state_clone.clone();
        let left_panel_for_timer = left_panel_clone.clone();
        let right_panel_for_timer = right_panel_clone.clone();
        let header_bar_for_timer = header_bar_clone.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(rx) = result_rx_for_timer.try_borrow() {
                if let Ok(msg) = rx.try_recv() {
                    let state = app_state_for_timer.clone();
                    let left_panel = left_panel_for_timer.clone();
                    let right_panel = right_panel_for_timer.clone();
                    let header_bar = header_bar_for_timer.clone();

                    match msg {
                        DatabaseResult::Backtest(result) => {
                            let button: gtk4::Button = utils::find_widget(&left_panel, "execute");
                            let progress_bar: gtk4::ProgressBar =
                                utils::find_widget(&left_panel, "progress");
                            let spinner: gtk4::Spinner =
                                utils::find_widget(&right_panel, "spinner");
                            match result {
                                Ok(mut rows) => {
                                    rows.sort_by(|a, b| {
                                        b.total_profit
                                            .partial_cmp(&a.total_profit)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    });
                                    let total_rows = rows.len();
                                    query::update_results_count(&header_bar, total_rows);
                                    state.borrow_mut().results = rows.clone();
                                    
                                    let batch_state = Rc::new(RefCell::new((rows, 0)));
                                    glib::idle_add_local(move || {
                                        let mut state_guard = batch_state.borrow_mut();
                                        let (all_rows, current_index) = &mut *state_guard;
                                        let end = (*current_index + BATCH_SIZE).min(all_rows.len());
                                        query::populate_store_batch(
                                            &state.borrow().store,
                                            &all_rows[*current_index..end],
                                        );
                                        *current_index = end;
                                        if *current_index < all_rows.len() {
                                            glib::ControlFlow::Continue
                                        } else {
                                            query::update_status(
                                                &right_panel,
                                                &format!(
                                                    "✅ {} resultados encontrados",
                                                    total_rows
                                                ),
                                            );
                                            query::enable_export_buttons(
                                                &header_bar,
                                                !all_rows.is_empty(),
                                            );
                                            state.borrow_mut().is_loading = false;
                                            button.set_sensitive(true);
                                            progress_bar.set_visible(false);
                                            spinner.stop();
                                            spinner.set_visible(false);
                                            glib::ControlFlow::Break
                                        }
                                    });
                                }
                                Err(e_string) => {
                                    query::update_status(
                                        &right_panel,
                                        &format!("❌ Error: {}", e_string),
                                    );
                                    state.borrow_mut().is_loading = false;
                                    button.set_sensitive(true);
                                    progress_bar.set_visible(false);
                                    spinner.stop();
                                    spinner.set_visible(false);
                                }
                            }
                        }
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    });

    window.present();
}

fn database_worker(
    command_rx: mpsc::Receiver<DatabaseCommand>,
    result_tx: mpsc::Sender<DatabaseResult>,
) {
    let rt = get_runtime();
    while let Ok(command) = command_rx.recv() {
        let result_tx = result_tx.clone();
        rt.spawn(async move {
            let client = db::get_db_pool()
                .get()
                .await
                .expect("Failed to get client from pool");
            match command {
                DatabaseCommand::RunBacktest(query) => {
                    let result = get_grid_summary(&client, &query)
                        .await
                        .map_err(|e| e.to_string());
                    result_tx.send(DatabaseResult::Backtest(result)).unwrap();
                }
            }
        });
    }
}
