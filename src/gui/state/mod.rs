use eframe::egui::Color32;

mod filter;
mod highlight_id;

use crate::config::{write_config, Config};
use crate::gui::MessageLoader;
use crate::util::remove_whitespace;

use self::filter::FilterLabelState;
pub(crate) use self::filter::{EditFilterLabelState, EditFilterOptionsState};
use self::highlight_id::HighlightIDState;

#[derive(Debug, Clone)]
pub struct ParseError {}

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
    pub fn as_string(&self, allow_empty: bool) -> Result<String, ParseError> {
        match allow_empty {
            true => Ok(self.value.clone()),
            false => {
                if self.value.is_empty() {
                    Err(ParseError {})
                } else {
                    Ok(self.value.clone())
                }
            }
        }
    }

    pub fn validate_string(&mut self, allow_empty: bool) -> Result<String, ParseError> {
        let result = self.as_string(allow_empty);
        self.valid = result.is_ok();
        result
    }

    pub fn as_bytes(&self, allow_empty: bool) -> Result<Vec<u8>, ParseError> {
        let mut value = self.value.clone();
        remove_whitespace(&mut value);

        let length_valid = match allow_empty {
            true => true,
            false => !value.is_empty(),
        };
        if !length_valid {
            return Err(ParseError {});
        }

        if value.len() % 2 != 0 {
            value.insert(0, '0');
        }

        hex::decode(&value).map_err(|_| ParseError {})
    }

    pub fn validate_bytes(&mut self, allow_empty: bool) -> Result<Vec<u8>, ParseError> {
        let result = self.as_bytes(allow_empty);
        self.valid = result.is_ok();
        result
    }
}

pub(crate) struct TableGui {
    pub message_loader: MessageLoader,
    pub highlight_id_state: HighlightIDState,
    pub filter_label_state: FilterLabelState,
}

impl TableGui {
    pub fn new() -> Self {
        Self {
            message_loader: MessageLoader::new(),
            highlight_id_state: HighlightIDState::default(),
            filter_label_state: FilterLabelState::default(),
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self {
            message_loader: match config.file_path {
                Some(path) => MessageLoader::from_path(path),
                None => MessageLoader::new(),
            },
            highlight_id_state: HighlightIDState::from_data(config.highlight_ids),
            filter_label_state: FilterLabelState::from_data(config.label_filters),
        }
    }

    pub fn save_state(&self) {
        let config = Config::new(
            self.message_loader.file_path().cloned(),
            self.highlight_id_state.data.clone(),
            self.filter_label_state.data.clone(),
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
