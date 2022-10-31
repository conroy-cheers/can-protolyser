#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod file;
mod filter;
mod gui;
mod label;
mod message;
mod util;

use eframe::egui;

use crate::config::read_config;
use crate::gui::TableGui;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        ..Default::default()
    };

    let app: TableGui;
    match read_config() {
        Ok(config) => {
            println!("Loaded config");
            app = TableGui::from_config(config);
        }
        Err(e) => {
            println!("Error reading config: {}", e);
            println!("Using defaults...");
            app = TableGui::new();
        }
    }

    eframe::run_native("Message Decoder", options, Box::new(|_cc| Box::new(app)));
}
