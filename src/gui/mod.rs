mod dialog;
mod message_loader;
mod state;
mod util;
mod widgets;

use std::collections::HashSet;

use crate::egui::{self, Align, Color32, ComboBox, Layout, ProgressBar, TextEdit};
use egui_extras::{Size, StripBuilder, TableBuilder};
use strum::IntoEnumIterator;

use crate::filter::{FilterType, OutputSelection};
use crate::message::{id_string, HighlightID, Message};
use crate::util::{bytes_to_string, hex_to_str};

pub(crate) use state::{EditFilterOptionsState, TableGui};

use self::dialog::csv_from_dialog;
use self::message_loader::{MessageLoader, MessageLoaderState};
use self::state::{EditFilterLabelState, Field};
use self::util::{ack_color, speed_color};
use self::widgets::{color_chip, colored_label};

pub fn id_text(id_field: &Field<String>, ids: &Vec<HighlightID>) -> String {
    match id_field.as_bytes(true) {
        Err(_) => id_field.value.clone(),
        Ok(id_bytes) => id_string(&id_bytes, ids),
    }
}

pub fn speed_text(speed_field: &Field<String>) -> String {
    match speed_field.as_string(true) {
        Err(_) => speed_field.value.clone(),
        Ok(speed) => match speed.is_empty() {
            true => "any".to_string(),
            false => speed.clone(),
        },
    }
}

impl eframe::App for TableGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.message_loader.handle_file_loading();

        egui::SidePanel::left("side_panel")
            .default_width(500.0)
            .show(ctx, |ui| {
                self.left_pane_ui(ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let response = ui.button("Open...");
                if response.clicked() {
                    match csv_from_dialog() {
                        Ok(Some(path)) => {
                            self.message_loader.replace_file_path(Some(path));
                            self.save_state();
                        }
                        Ok(None) => {} // User cancelled
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
                        self.messages_ui(ui);
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
                strip.cell(|ui| {
                    self.labels_ui(ui);
                });
            });
    }

    fn labels_table_ui(&mut self, ui: &mut egui::Ui) {
        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::relative(0.3))
            .column(Size::initial(50.0).at_least(30.0))
            .column(Size::initial(50.0).at_least(30.0))
            .column(Size::remainder())
            .column(Size::exact(150.0));

        let mut index_to_remove: Option<usize> = None;
        let mut index_to_copy: Option<usize> = None;

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Label");
                });
                header.col(|ui| {
                    ui.heading("ID");
                });
                header.col(|ui| {
                    ui.heading("Speed");
                });
                header.col(|ui| {
                    ui.heading("Rule");
                });
                header.col(|_| {});
            })
            .body(|body| {
                body.rows(
                    TableGui::BUTTON_HEIGHT,
                    self.filter_label_state.data.len(),
                    |row_index, mut row| {
                        let label_filter = &self.filter_label_state.data[row_index];
                        let filter = &label_filter.filter;

                        match self.filter_label_state.editing_index() {
                            None => {
                                // No row being edited
                                row.col(|ui| {
                                    colored_label(
                                        ui,
                                        label_filter.label.color32(),
                                        &label_filter.label.name,
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        &label_filter
                                            .filter
                                            .id_string(&self.highlight_id_state.data),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(&label_filter.filter.speed_string());
                                });
                                row.col(|ui| {
                                    ui.label(filter.description());
                                });
                                row.col(|ui| {
                                    if ui.button("Edit").clicked() {
                                        self.filter_label_state.edit(row_index);
                                    }
                                    if ui.button("Copy").clicked() {
                                        index_to_copy = Some(row_index);
                                    }
                                    if ui.button("Delete").clicked() {
                                        index_to_remove = Some(row_index);
                                    }
                                });
                            }
                            Some(editing_index) if *editing_index == row_index => {
                                // Empty row
                                row.col(|ui| {
                                    ui.label("editing...");
                                });
                            }
                            Some(_) => {
                                // Another row being edited
                                row.col(|ui| {
                                    colored_label(
                                        ui,
                                        label_filter.label.color32(),
                                        &label_filter.label.name,
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        &label_filter
                                            .filter
                                            .id_string(&self.highlight_id_state.data),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(&label_filter.filter.speed_string());
                                });
                                row.col(|ui| {
                                    ui.label(filter.description());
                                });
                            }
                        }
                    },
                );
            });

        if index_to_remove.is_some() {
            self.filter_label_state
                .data
                .remove(index_to_remove.unwrap());
            self.save_state();
        }
        if index_to_copy.is_some() {
            let index = index_to_copy.unwrap();
            let label_filter = self.filter_label_state.data[index].clone();
            self.filter_label_state.data.insert(index + 1, label_filter);
            self.save_state();
        }
    }

    fn edit_label_ui(&mut self, ui: &mut egui::Ui) {
        let rule_select_text = &self
            .filter_label_state
            .edit_state
            .filter_type
            .name()
            .to_owned();

        ui.horizontal(|ui| {
            ui.label("Label:");
            ui.add(
                TextEdit::singleline(&mut self.filter_label_state.edit_state.name.value)
                    .desired_width(80.0)
                    .text_color_opt(match self.filter_label_state.edit_state.name.valid {
                        true => None,
                        false => Some(Color32::RED),
                    }),
            );
            ui.color_edit_button_rgb(&mut self.filter_label_state.edit_state.color.value);
            ui.label("Rule:");
            ComboBox::from_id_source("add_label_rule")
                .selected_text(rule_select_text)
                .show_ui(ui, |ui| {
                    for rule in FilterType::iter() {
                        if ui
                            .selectable_label(
                                self.filter_label_state
                                    .edit_state
                                    .filter_type
                                    .is_variant(&rule),
                                rule.name(),
                            )
                            .clicked()
                        {
                            self.filter_label_state.edit_state.filter_options =
                                EditFilterOptionsState::from_filter_type(&rule);
                            self.filter_label_state.edit_state.filter_type = rule;
                        }
                    }
                });
        });
        match (
            &self.filter_label_state.edit_state.filter_type,
            &mut self.filter_label_state.edit_state.filter_options,
        ) {
            (FilterType::Basic, EditFilterOptionsState::Empty) => {
                TableGui::basic_filter_edit_line(
                    ui,
                    &mut self.filter_label_state.edit_state.id,
                    &mut self.filter_label_state.edit_state.speed,
                    &self.highlight_id_state.data,
                    self.message_loader.known_speeds(),
                );
            }
            (
                FilterType::StartsWithBytes(_),
                EditFilterOptionsState::OneStringFieldOneOutputSelection(field, output),
            ) => {
                TableGui::basic_filter_edit_line(
                    ui,
                    &mut self.filter_label_state.edit_state.id,
                    &mut self.filter_label_state.edit_state.speed,
                    &self.highlight_id_state.data,
                    self.message_loader.known_speeds(),
                );
                TableGui::one_string_edit_line(ui, &"Starts with".to_string(), field);
                TableGui::output_selection_edit_line(ui, output);
            }
            _ => {}
        }

        ui.horizontal(|ui| match &self.filter_label_state.editing_index() {
            Some(_) => {
                if ui.button("Save").clicked() {
                    self.filter_label_state.commit();
                    self.save_state();
                }
            }
            None => {
                if ui.button("Add").clicked() {
                    match self.filter_label_state.edit_state.validate() {
                        Ok(filter) => {
                            self.filter_label_state.data.push(filter);
                            self.filter_label_state.edit_state = EditFilterLabelState::default();
                            self.save_state();
                        }
                        Err(_) => {}
                    }
                }
            }
        });
    }

    fn basic_filter_edit_line(
        ui: &mut egui::Ui,
        id_field: &mut Field<String>,
        speed_field: &mut Field<String>,
        highlight_ids: &Vec<HighlightID>,
        known_speeds: &HashSet<String>,
    ) {
        let current_id_data = id_field.validate_bytes(false);

        ui.horizontal(|ui| {
            ui.label("ID:");
            ComboBox::from_id_source("add_label_id")
                .selected_text(id_text(id_field, highlight_ids))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(id_field.value.is_empty(), "any")
                        .clicked()
                    {
                        id_field.value = "".to_string();
                    }
                    for h_id in highlight_ids {
                        if ui
                            .selectable_label(
                                match current_id_data {
                                    Ok(ref id_data) => id_data.clone() == h_id.id().clone(),
                                    Err(_) => false,
                                },
                                h_id.name(),
                            )
                            .clicked()
                        {
                            id_field.value = hex_to_str(h_id.id());
                        }
                    }
                });

            ui.label("Speed:");
            ComboBox::from_id_source("add_label_speed")
                .selected_text(speed_text(speed_field))
                .show_ui(ui, |ui| {
                    for speed in known_speeds {
                        if ui
                            .selectable_label(speed_field.value == speed.clone(), speed.to_string())
                            .clicked()
                        {
                            speed_field.value = speed.clone();
                        }
                    }
                });
        });
    }

    fn one_string_edit_line(ui: &mut egui::Ui, label: &String, field: &mut Field<String>) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.text_edit_singleline(&mut field.value);
        });
    }

    fn output_selection_edit_line(ui: &mut egui::Ui, output: &mut OutputSelection) {
        ui.horizontal(|ui| {
            ui.label("Output:");
            ComboBox::from_id_source("add_label_output")
                .selected_text(output.name())
                .show_ui(ui, |ui| {
                    for output_type in OutputSelection::iter() {
                        if ui
                            .selectable_label(*output == output_type, output_type.name())
                            .clicked()
                        {
                            *output = output_type;
                        }
                    }
                });
        });
    }

    fn labels_ui(&mut self, ui: &mut egui::Ui) {
        ui.push_id("labels_ui", |ui| {
            StripBuilder::new(ui)
                .size(Size::relative(0.5))
                .size(Size::relative(0.5))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.labels_table_ui(ui);
                    });
                    strip.cell(|ui| {
                        self.edit_label_ui(ui);
                    });
                });
        });
    }

    fn id_config_add_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("ID:");
            let id_field = ui.add(
                TextEdit::singleline(&mut self.highlight_id_state.edit_state.id)
                    .desired_width(80.0)
                    .text_color_opt(match self.highlight_id_state.edit_state.id_valid {
                        true => None,
                        false => Some(Color32::RED),
                    }),
            );
            ui.label("Name:");
            let name_field = ui.add(
                TextEdit::singleline(&mut self.highlight_id_state.edit_state.name)
                    .desired_width(70.0)
                    .text_color_opt(match self.highlight_id_state.edit_state.name_valid {
                        true => None,
                        false => Some(Color32::RED),
                    }),
            );
            if id_field.changed() || name_field.changed() {
                self.highlight_id_state.edit_state.clear_validation();
            }

            ui.label("Color:");
            ui.color_edit_button_rgb(&mut self.highlight_id_state.edit_state.color);
            match &self.highlight_id_state.editing_index() {
                Some(_) => {
                    if ui.button("Save").clicked() {
                        self.highlight_id_state.commit();
                        self.save_state();
                    }
                }
                None => {
                    if ui.button("Add").clicked() {
                        match (
                            self.highlight_id_state.edit_state.validate_id(),
                            self.highlight_id_state.edit_state.validate_name(),
                        ) {
                            (Some(id), Some(name)) => {
                                self.highlight_id_state.data.push(HighlightID::new(
                                    id,
                                    name,
                                    self.highlight_id_state.edit_state.color,
                                ));
                                self.highlight_id_state.edit_state.clear();
                                self.save_state();
                            }
                            (id_result, name_result) => {
                                self.highlight_id_state.edit_state.id_valid = id_result.is_some();
                                self.highlight_id_state.edit_state.name_valid =
                                    name_result.is_some();
                            }
                        }
                    }
                }
            }
        });
    }

    fn id_config_table(&mut self, ui: &mut egui::Ui) {
        ui.push_id("id_config_table", |ui| {
            ui.vertical(|ui| {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Size::remainder())
                    .column(Size::remainder())
                    .column(Size::initial(70.0).at_least(30.0))
                    .column(Size::exact(110.0));

                let mut index_to_remove: Option<usize> = None;

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
                            self.highlight_id_state.data.len(),
                            |row_index, mut row| {
                                match *self.highlight_id_state.editing_index() {
                                    None => {
                                        // No row being edited
                                        let msg = &mut self.highlight_id_state.data[row_index];
                                        row.col(|ui| {
                                            ui.label(hex_to_str(msg.id()));
                                        });
                                        row.col(|ui| {
                                            ui.label(msg.name());
                                        });
                                        row.col(|ui| {
                                            color_chip(ui, msg.color32());
                                        });
                                        row.col(|ui| {
                                            if ui.button("Edit").clicked() {
                                                self.highlight_id_state.edit(row_index);
                                            }
                                            if ui.button("Delete").clicked() {
                                                index_to_remove = Some(row_index);
                                            }
                                        });
                                    }
                                    i if i == Some(row_index) => {
                                        // Empty row
                                        row.col(|ui| {
                                            ui.label("editing...");
                                        });
                                    }
                                    _ => {
                                        // Row while another is being edited
                                        let msg = &mut self.highlight_id_state.data[row_index];
                                        row.col(|ui| {
                                            ui.label(hex_to_str(msg.id()));
                                        });
                                        row.col(|ui| {
                                            ui.label(msg.name());
                                        });
                                        row.col(|ui| {
                                            color_chip(ui, msg.color32());
                                        });
                                        row.col(|_| {});
                                    }
                                }
                            },
                        );
                    });
                if index_to_remove.is_some() {
                    self.highlight_id_state
                        .data
                        .remove(index_to_remove.unwrap());
                    self.save_state();
                }
            });
        });
    }

    fn message_loading_ui(&self, ui: &mut egui::Ui, label: &String) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(20.0);
            ui.add(
                ProgressBar::new(self.message_loader.loading_progress())
                    .desired_width(150.0)
                    .animate(true),
            );
            ui.label(label);
        });
    }

    fn table_from_messages_ui(&self, ui: &mut egui::Ui, messages: &Vec<Message>) {
        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::initial(70.0).at_least(30.0))
            .column(Size::initial(80.0).at_least(40.0))
            .column(Size::initial(160.0).at_least(90.0))
            .column(Size::initial(80.0).at_least(90.0))
            .columns(Size::initial(40.0).at_least(40.0), 2)
            .column(Size::initial(50.0).at_least(40.0))
            .column(Size::remainder().at_least(60.0))
            .resizable(false);

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
                    ui.heading("Text");
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
                header.col(|ui| {
                    ui.heading("Labels");
                });
            })
            .body(|body| {
                body.rows(
                    TableGui::BUTTON_HEIGHT,
                    messages.len(),
                    |row_index, mut row| {
                        let msg = &messages[row_index];
                        row.col(|ui| {
                            ui.label(std::format!("{:.3}", msg.timestamp));
                        });
                        row.col(|ui| match msg.match_id(&self.highlight_id_state.data) {
                            Some(id) => {
                                ui.colored_label(id.color32(), id.name());
                            }
                            None => {
                                ui.label(hex_to_str(&msg.id));
                            }
                        });
                        row.col(|ui| {
                            ui.label(hex_to_str(&msg.data));
                        });
                        row.col(|ui| {
                            ui.label(bytes_to_string(&msg.data));
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
                        row.col(|ui| {
                            self.filter_label_state
                                .matching_labels(msg)
                                .iter()
                                .for_each(|result| match &result.output {
                                    Some(data) => {
                                        colored_label(
                                            ui,
                                            result.label.color32(),
                                            &(result.label.name.clone()
                                                + ": "
                                                + &hex_to_str(&data)),
                                        );
                                    }
                                    None => {
                                        colored_label(
                                            ui,
                                            result.label.color32(),
                                            &result.label.name,
                                        );
                                    }
                                });
                        });
                    },
                );
            });
    }

    fn messages_ui(&self, ui: &mut egui::Ui) {
        match &self.message_loader.state() {
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
