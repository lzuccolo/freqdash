use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::gui_adw::window::AppWindow;
use crate::{config, db};
use crate::backtest::model::{GridQuery, StrategyGridRow};

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get().expect("Runtime not initialized")
}

#[derive(Debug, Clone)]
pub enum DatabaseCommand {
    FetchPairlist,
    RunBacktest(u64, GridQuery),
}

#[derive(Debug)]
pub enum DatabaseResult {
    Pairlist(Result<Vec<String>, String>),
    Backtest(u64, Result<Vec<StrategyGridRow>, String>),
}

pub fn run() {
    config::init_config();
    let rt = Runtime::new().unwrap();
    TOKIO_RUNTIME.set(rt).unwrap();
    db::init_db_pool();

    let (command_tx, mut command_rx) = mpsc::channel::<DatabaseCommand>(32);
    let (result_tx, result_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

    get_runtime().spawn(async move {
        let pool = db::get_db_pool();
        while let Some(command) = command_rx.recv().await {
            let pool = pool.clone();
            let result_tx = result_tx.clone();
            tokio::spawn(async move {
                let client = pool.get().await.expect("Failed to get client");
                match command {
                    DatabaseCommand::FetchPairlist => {
                        let res = async {
                            let rows = client.query("SELECT DISTINCT pairlist FROM backtest WHERE pairlist IS NOT NULL", &[]).await?;
                            let pairs: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
                            Ok(pairs)
                        }.await;
                        let _ = result_tx.send(DatabaseResult::Pairlist(res.map_err(|e: tokio_postgres::Error| e.to_string())));
                    },
                    DatabaseCommand::RunBacktest(id, query) => {
                        let res = crate::backtest::logic::get_grid_summary(&client, &query).await;
                        let _ = result_tx.send(DatabaseResult::Backtest(id, res.map_err(|e| e.to_string())));
                    }
                }
            });
        }
    });

    let app = adw::Application::new(Some("com.example.freqdash"), Default::default());

    app.connect_activate(move |app| {
        let window = AppWindow::new(app, command_tx.clone());
        
        result_rx.attach(None, clone!(@weak window => @default-return glib::ControlFlow::Break, move |msg| {
            match msg {
                DatabaseResult::Backtest(tab_id, result_data) => {
                    window.route_result(tab_id, result_data);
                },
                DatabaseResult::Pairlist(Ok(pairs)) => {
                    window.update_all_pairlists(pairs);
                },
                _ => {}
            }
            glib::ControlFlow::Continue
        }));
        
        window.present();
    });
    app.run();
}