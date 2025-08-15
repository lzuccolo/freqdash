// src/bin/gui.rs

#![cfg(feature = "gui")]
use freqdash::gui;

fn main() {
    // Llamamos a la función `run` desde su nuevo hogar
    gui::app::run();
}