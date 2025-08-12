pub mod backtest;
pub mod config;
pub mod db;
pub mod utils;

#[cfg(feature = "gtk")]
pub mod gui;

#[cfg(feature = "adwaita")]
pub mod gui_adw;