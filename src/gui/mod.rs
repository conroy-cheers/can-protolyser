mod dialog;
mod message_loader;
mod state;
mod util;

use eframe::egui::{self, Align, Color32, Layout, ProgressBar, TextEdit};
use egui_extras::{Size, StripBuilder, TableBuilder};

use crate::message::{HighlightID, Message};
use crate::util::hex_to_str;

use dialog::csv_from_dialog;
use message_loader::{MessageLoader, MessageLoaderState};
pub use state::TableGui;
use util::{ack_color, speed_color};

impl eframe::App for TableGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.message_loader.handle_file_loading();

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            self.left_pane_ui(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let response = ui.button("Open...");
                if response.clicked() {
                    match csv_from_dialog() {
                        Ok(path) => {
                            self.message_loader.replace_file_path(path);
                            self.save_state();
                        }
                        Err(e) => {
                            self.message_loader.set_error(e.to_string());
                        }
                    }
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

    fn message_loading_ui(&self, ui: &mut egui::Ui, label: &String) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(20.0);
            ui.add(
                ProgressBar::new(self.message_loader.get_loading_progress())
                    .desired_width(150.0)
                    .animate(true),
            );
            ui.label(label);
        });
    }

    fn table_from_messages_ui(&self, ui: &mut egui::Ui, messages: &Vec<Message>) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::initial(70.0).at_least(30.0))
            .column(Size::initial(80.0).at_least(40.0))
            .column(Size::initial(160.0).at_least(90.0))
            .columns(Size::initial(40.0).at_least(40.0), 2)
            .column(Size::remainder().at_least(60.0))
            .resizable(true);

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
                body.rows(text_height, messages.len(), |row_index, mut row| {
                    let msg = &messages[row_index];
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

    fn table_ui(&self, ui: &mut egui::Ui) {
        match &self.message_loader.state {
            MessageLoaderState::FileNotSelected => {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(20.0);
                    ui.label("No file loaded");
                });
            }
            MessageLoaderState::FileSelected(file_path)
            | MessageLoaderState::Loading { file_path, .. } => {
                let loading_msg = format!(
                    "Loading {}...",
                    file_path.file_name().unwrap().to_str().unwrap()
                );
                self.message_loading_ui(ui, &loading_msg);
            }
            MessageLoaderState::Loaded { messages, .. } => {
                self.table_from_messages_ui(ui, &messages);
            }
            MessageLoaderState::Error { file_path, error } => {
                let error_msg = match file_path {
                    Some(file_path) => {
                        format!(
                            "Error loading {}",
                            file_path.file_name().unwrap().to_str().unwrap(),
                        )
                    }
                    None => "Error loading file".to_string(),
                };
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(20.0);
                    ui.label(error_msg);
                    if error.is_some() {
                        ui.label(error.as_ref().unwrap());
                    }
                });
            }
        };
    }
}
