#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use clap::Parser;

mod dialog;
mod file;
mod gui;
mod message;
mod util;

use eframe::egui;

use crate::gui::TableGui;
use crate::util::read_config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opt {
    #[arg(help = "The CSV message file to parse")]
    file: String,
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(900.0, 800.0)),
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
