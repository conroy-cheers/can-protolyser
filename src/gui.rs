use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use crate::dialog::csv_from_dialog;
use crate::file::load_file_async;
use crate::message::{HighlightID, Message};
use crate::util::{hex_to_str, write_config};
use eframe::egui::{self, Align, Color32, Layout, ProgressBar, TextEdit};
use egui_extras::{Size, StripBuilder, TableBuilder};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Config {
    file_path: Option<PathBuf>,
    highlight_ids: Vec<HighlightID>,
}

impl Config {
    pub fn new(file_path: Option<PathBuf>, highlight_ids: Vec<HighlightID>) -> Self {
        Self {
            file_path,
            highlight_ids,
        }
    }
}

pub struct TableGui {
    file_path: Option<PathBuf>,
    vertical_scroll_offset: Option<f32>,
    messages: Vec<Message>,
    messages_loading: Option<(Sender<Option<Vec<Message>>>, Receiver<Option<Vec<Message>>>)>,
    messages_loading_progress: Arc<Mutex<f32>>,
    messages_loaded: bool,
    highlight_ids: Vec<HighlightID>,
    add_highlight_id_state: AddHighlightIDState,
}

struct AddHighlightIDState {
    id: String,
    id_valid: bool,
    name: String,
    name_valid: bool,
    color: [f32; 3],
}

impl AddHighlightIDState {
    fn new() -> Self {
        Self {
            id: String::new(),
            id_valid: true,
            name: String::new(),
            name_valid: true,
            color: [255.0, 255.0, 255.0],
        }
    }

    fn clear(&mut self) {
        self.id.clear();
        self.id_valid = true;
        self.name.clear();
        self.name_valid = true;
        self.color = [255.0, 255.0, 255.0];
    }

    fn validate_id(&mut self) -> Option<Vec<u8>> {
        if self.id.len() == 0 {
            self.id_valid = false;
            return None;
        }
        if self.id.len() % 2 != 0 {
            self.id.insert(0, '0');
        }

        match hex::decode(&self.id) {
            Ok(id) => Some(id),
            Err(_) => {
                self.id_valid = false;
                return None;
            }
        }
    }

    fn validate_name(&mut self) -> Option<String> {
        if self.name.len() == 0 {
            self.name_valid = false;
            return None;
        }
        Some(self.name.clone())
    }

    fn clear_validation(&mut self) {
        self.id_valid = true;
        self.name_valid = true;
    }
}

impl HighlightID {
    fn color32(&self) -> Color32 {
        Color32::from_rgb(
            (self.color[0] * 255.0) as u8,
            (self.color[1] * 255.0) as u8,
            (self.color[2] * 255.0) as u8,
        )
    }
}

impl TableGui {
    pub fn new() -> Self {
        Self {
            file_path: None,
            vertical_scroll_offset: None,
            messages: vec![],
            messages_loading: None,
            messages_loading_progress: Arc::new(Mutex::new(0.0)),
            messages_loaded: false,
            highlight_ids: vec![],
            add_highlight_id_state: AddHighlightIDState::new(),
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self {
            file_path: config.file_path,
            vertical_scroll_offset: None,
            messages: vec![],
            messages_loading: None,
            messages_loading_progress: Arc::new(Mutex::new(0.0)),
            messages_loaded: false,
            highlight_ids: config.highlight_ids,
            add_highlight_id_state: AddHighlightIDState::new(),
        }
    }
}

fn ack_color(ack: bool) -> Color32 {
    if ack {
        Color32::GREEN
    } else {
        Color32::RED
    }
}

fn speed_color(speed: &String) -> Color32 {
    match speed.as_str() {
        "1M" => Color32::GREEN,
        "667k" => Color32::YELLOW,
        "500k" => Color32::RED,
        _ => Color32::WHITE,
    }
}

fn scroll_offset_for_row(ui: &egui::Ui, row: i32) -> f32 {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
    let row_item_spacing = ui.spacing().item_spacing.y;
    row as f32 * (text_height + row_item_spacing)
}

impl eframe::App for TableGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_file_loading();

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            self.left_pane_ui(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let response = ui.button("Open...");
                if response.clicked() {
                    self.unset_file_path();
                    self.file_path = csv_from_dialog();
                    return;
                }
            });

            ui.separator();

            // Leave room for the source code link after the table demo:
            StripBuilder::new(ui)
                .size(Size::remainder()) // for the table
                .size(Size::exact(10.0)) // for the source code link
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.table_ui(ui);
                    });
                });
        });
    }
}

impl TableGui {
    const BUTTON_HEIGHT: f32 = 20.0;

    fn save_state(&mut self) {
        let config = Config::new(self.file_path.clone(), self.highlight_ids.clone());
        match write_config(&config) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error saving config: {}", e);
            }
        }
    }

    fn unset_file_path(&mut self) {
        self.file_path = None;
        self.messages_loading = None;
        self.messages_loaded = false;
    }

    fn get_loading_position(&self) -> f32 {
        self.messages_loading_progress.lock().unwrap().clone()
    }

    fn handle_file_loading(&mut self) {
        if self.file_path.is_none() {
            return;
        }

        if self.messages_loaded {
            return;
        }

        // Valid file path but not loaded yet

        match &self.messages_loading {
            None => {
                // Start loading
                self.messages_loading = Some(std::sync::mpsc::channel());
                *self.messages_loading_progress.lock().unwrap() = 0.0;
                let (tx, _rx) = self.messages_loading.as_ref().unwrap();
                load_file_async(
                    self.file_path.as_ref().unwrap(),
                    self.messages_loading_progress.clone(),
                    tx.to_owned(),
                );
            }
            Some((_tx, rx)) => {
                // Update progress
                match rx.try_recv() {
                    Err(TryRecvError::Empty) => {} // Still loading
                    Err(_) => {
                        // Something went wrong
                        self.messages_loading = None;
                    }
                    Ok(Some(messages)) => {
                        self.messages = messages;
                        self.messages_loaded = true;
                        self.messages_loading = None;
                    }
                    Ok(None) => {
                        // Load failed
                        self.messages_loading = None;
                    }
                }
            }
        }
    }

    fn left_pane_ui(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::relative(0.3))
            .size(Size::exact(TableGui::BUTTON_HEIGHT)) // for the "Add" button
            .size(Size::exact(5.0)) // for the separator
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    self.id_config_table(ui);
                });
                strip.cell(|ui| {
                    self.id_config_add_controls(ui);
                });
                strip.cell(|ui| {
                    ui.separator();
                });
                strip.cell(|_ui| {});
            });
    }

    fn id_config_add_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("ID:");
            let id_field = ui.add(
                TextEdit::singleline(&mut self.add_highlight_id_state.id)
                    .desired_width(80.0)
                    .text_color_opt(match self.add_highlight_id_state.id_valid {
                        true => None,
                        false => Some(Color32::RED),
                    }),
            );
            ui.label("Name:");
            let name_field = ui.add(
                TextEdit::singleline(&mut self.add_highlight_id_state.name)
                    .desired_width(70.0)
                    .text_color_opt(match self.add_highlight_id_state.name_valid {
                        true => None,
                        false => Some(Color32::RED),
                    }),
            );
            if id_field.changed() || name_field.changed() {
                self.add_highlight_id_state.clear_validation();
            }

            ui.label("Color:");
            ui.color_edit_button_rgb(&mut self.add_highlight_id_state.color);
            if ui.button("Add").clicked() {
                match (
                    self.add_highlight_id_state.validate_id(),
                    self.add_highlight_id_state.validate_name(),
                ) {
                    (Some(id), Some(name)) => {
                        self.highlight_ids.push(HighlightID {
                            id,
                            name,
                            color: self.add_highlight_id_state.color,
                        });
                        self.add_highlight_id_state.clear();
                        self.save_state();
                    }
                    (id_result, name_result) => {
                        self.add_highlight_id_state.id_valid = id_result.is_some();
                        self.add_highlight_id_state.name_valid = name_result.is_some();
                    }
                }
            }
        });
    }

    fn id_config_table(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::remainder())
                .column(Size::remainder())
                .column(Size::initial(70.0).at_least(30.0))
                .column(Size::initial(50.0).at_least(30.0));

            let mut id_to_remove: Option<usize> = None;

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("ID");
                    });
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Colour");
                    });
                    header.col(|_ui| {});
                })
                .body(|body| {
                    body.rows(
                        TableGui::BUTTON_HEIGHT,
                        self.highlight_ids.len(),
                        |row_index, mut row| {
                            let msg = &mut self.highlight_ids[row_index];
                            row.col(|ui| {
                                ui.label(hex_to_str(&msg.id));
                            });
                            row.col(|ui| {
                                ui.label(&msg.name);
                            });
                            row.col(|ui| {
                                ui.color_edit_button_rgb(&mut msg.color);
                            });
                            row.col(|ui| {
                                if ui.button("Delete").clicked() {
                                    id_to_remove = Some(row_index);
                                }
                            });
                        },
                    );
                });

            if id_to_remove != None {
                self.highlight_ids.remove(id_to_remove.unwrap());
                self.save_state();
            }
        });
    }

    fn table_ui(&mut self, ui: &mut egui::Ui) {
        if self.file_path.is_none() {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(20.0);
                ui.label("No file loaded");
            });
            return;
        }

        match self.messages_loaded {
            false => {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(20.0);
                    ui.add(
                        ProgressBar::new(self.get_loading_position())
                            .desired_width(150.0)
                            .animate(true),
                    );
                    ui.label(match &self.file_path {
                        Some(path) => {
                            format!("Loading {}...", path.file_name().unwrap().to_str().unwrap())
                        }
                        None => "Loading...".to_string(),
                    });
                });
            }
            true => {
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Size::initial(70.0).at_least(30.0))
                    .column(Size::initial(80.0).at_least(40.0))
                    .column(Size::initial(160.0).at_least(90.0))
                    .columns(Size::initial(40.0).at_least(40.0), 2)
                    .column(Size::remainder().at_least(60.0))
                    .resizable(true);

                if let Some(y_scroll) = self.vertical_scroll_offset.take() {
                    table = table.vertical_scroll_offset(y_scroll);
                }

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Time");
                        });
                        header.col(|ui| {
                            ui.heading("ID");
                        });
                        header.col(|ui| {
                            ui.heading("Data");
                        });
                        header.col(|ui| {
                            ui.heading("CRC");
                        });
                        header.col(|ui| {
                            ui.heading("ACK");
                        });
                        header.col(|ui| {
                            ui.heading("Speed");
                        });
                    })
                    .body(|body| {
                        body.rows(text_height, self.messages.len(), |row_index, mut row| {
                            let msg = &self.messages[row_index];
                            row.col(|ui| {
                                ui.label(std::format!("{:.3}", msg.timestamp));
                            });
                            row.col(|ui| match msg.match_id(&self.highlight_ids) {
                                Some(id) => {
                                    ui.colored_label(id.color32(), &id.name);
                                }
                                None => {
                                    ui.label(hex_to_str(&msg.id));
                                }
                            });
                            row.col(|ui| {
                                ui.label(hex_to_str(&msg.data));
                            });
                            row.col(|ui| {
                                ui.label(hex_to_str(&msg.crc));
                            });
                            row.col(|ui| {
                                ui.colored_label(ack_color(msg.ack), msg.ack.to_string());
                            });
                            row.col(|ui| {
                                ui.colored_label(speed_color(&msg.speed), msg.speed.to_string());
                            });
                        });
                    });
            }
        }
    }
}
