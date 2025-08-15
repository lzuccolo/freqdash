// src/bin/gui_adw.rs

#![cfg(feature = "adwaita")]
use freqdash::gui_adw;

fn main() {
    // Llamamos a la función `run` desde su nuevo hogar
    gui_adw::app::run();
}