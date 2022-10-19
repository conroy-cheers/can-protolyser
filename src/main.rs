#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::error::Error;
use std::process;

use clap::Parser;
use indicatif::ProgressBar;

mod gui;
mod message;
mod util;

use eframe::egui;

use crate::gui::TableGui;
use crate::message::Message;
use crate::util::read_config;

fn parse_file(path: String) -> Result<Vec<Message>, Box<dyn Error>> {
    let reader = csv::Reader::from_path(path.as_str())?;
    let num_lines = reader.into_records().count();
    let pb = ProgressBar::new(num_lines.try_into().unwrap());

    let mut msgs = Vec::with_capacity(num_lines);

    let mut reader = csv::Reader::from_path(path)?;
    for result in reader.deserialize() {
        pb.inc(1);
        msgs.push(result?);
    }
    pb.finish();
    Ok(msgs)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opt {
    #[arg(help = "The CSV message file to parse")]
    file: String,
}

fn main() {
    let opt = Opt::parse();

    println!("Loading messages from {}...", opt.file);
    match parse_file(opt.file) {
        Ok(msgs) => {
            let options = eframe::NativeOptions {
                initial_window_size: Some(egui::vec2(900.0, 800.0)),
                ..Default::default()
            };

            let app: TableGui;
            match read_config() {
                Ok(config) => {
                    println!("Loaded config");
                    app = TableGui::from_config(config, msgs);
                }
                Err(e) => {
                    println!("Error reading config: {}", e);
                    println!("Using defaults...");
                    app = TableGui::new(msgs);
                }
            }
            
            eframe::run_native(
                "Message Decoder",
                options,
                Box::new(|_cc| Box::new(app)),
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
