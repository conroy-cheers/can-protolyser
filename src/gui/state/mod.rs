use eframe::egui::Color32;

mod filter;
mod highlight_id;

use crate::config::{write_config, Config};
use crate::filter::LabelFilter;
use crate::gui::MessageLoader;
use crate::util::remove_whitespace;

pub(crate) use self::filter::{AddFilterOptionsState, FilterLabelEditState};
use self::highlight_id::HighlightIDState;

pub struct Field<T> {
    pub value: T,
    pub valid: bool,
}

impl<T> Field<T> {
    pub fn with_value(value: T) -> Self {
        Self { value, valid: true }
    }
}

impl<T> Default for Field<T>
where
    T: Default,
{
    fn default() -> Field<T> {
        Field {
            value: T::default(),
            valid: true,
        }
    }
}

impl Field<[f32; 3]> {
    pub fn color32(&self) -> Color32 {
        Color32::from_rgb(
            (self.value[0] * 255.0) as u8,
            (self.value[1] * 255.0) as u8,
            (self.value[2] * 255.0) as u8,
        )
    }
}

impl Field<String> {
    pub fn as_string(&self, allow_empty: bool) -> Option<String> {
        match allow_empty {
            true => Some(self.value.clone()),
            false => {
                if self.value.is_empty() {
                    None
                } else {
                    Some(self.value.clone())
                }
            }
        }
    }

    pub fn validate_string(&mut self, allow_empty: bool) -> Option<String> {
        match self.as_string(allow_empty) {
            None => {
                self.valid = false;
                None
            }
            Some(value) => {
                self.valid = true;
                Some(value)
            }
        }
    }

    pub fn as_bytes(&self, allow_empty: bool) -> Option<Vec<u8>> {
        let mut value = self.value.clone();
        remove_whitespace(&mut value);

        let length_valid = match allow_empty {
            true => true,
            false => !value.is_empty(),
        };
        if !length_valid {
            return None;
        }

        if value.len() % 2 != 0 {
            value.insert(0, '0');
        }

        match hex::decode(&value) {
            Ok(id) => Some(id),
            Err(_) => {
                return None;
            }
        }
    }

    pub fn validate_bytes(&mut self, allow_empty: bool) -> Option<Vec<u8>> {
        match self.as_bytes(allow_empty) {
            None => {
                self.valid = false;
                None
            }
            Some(value) => {
                self.valid = true;
                Some(value)
            }
        }
    }
}

pub(crate) struct TableGui {
    pub message_loader: MessageLoader,
    pub highlight_id_state: HighlightIDState,
    pub label_filters: Vec<LabelFilter>,
    pub edit_filter_state: FilterLabelEditState,
}

impl TableGui {
    pub fn new() -> Self {
        Self {
            message_loader: MessageLoader::new(),
            highlight_id_state: HighlightIDState::default(),
            label_filters: vec![],
            edit_filter_state: FilterLabelEditState::new(),
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self {
            message_loader: match config.file_path {
                Some(path) => MessageLoader::from_path(path),
                None => MessageLoader::new(),
            },
            highlight_id_state: HighlightIDState::from_data(config.highlight_ids),
            label_filters: config.label_filters,
            edit_filter_state: FilterLabelEditState::new(),
        }
    }

    pub fn save_state(&self) {
        let config = Config::new(
            self.message_loader.file_path().cloned(),
            self.highlight_id_state.data.clone(),
            self.label_filters.clone(),
        );
        match write_config(&config) {
            Ok(_) => {
                println!("Wrote config");
            }
            Err(e) => {
                eprintln!("Error saving config: {}", e);
            }
        }
    }
}
