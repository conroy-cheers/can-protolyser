use strum::EnumIter;

use crate::label::Label;
use crate::message::{id_string, HighlightID, Message, Speed};

pub trait SpecialFilter {
    fn filter_specific(&self, message: &Message) -> bool;

    fn output_data(&self, _message: &Message) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Debug, EnumIter, PartialEq, serde::Serialize, serde::Deserialize, Default, Clone)]
pub enum OutputSelection {
    All,
    #[default]
    AfterMatch,
}

impl OutputSelection {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            OutputSelection::All => "Whole message",
            OutputSelection::AfterMatch => "After match",
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct StartsWithBytes {
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub output: OutputSelection,
}

impl SpecialFilter for StartsWithBytes {
    fn filter_specific(&self, message: &Message) -> bool {
        message.data.starts_with(&self.bytes)
    }

    fn output_data(&self, _message: &Message) -> Option<Vec<u8>> {
        match self.output {
            OutputSelection::All => Some(_message.data.clone()),
            OutputSelection::AfterMatch => {
                if let Some(index) = _message
                    .data
                    .windows(self.bytes.len())
                    .position(|w| w == &self.bytes)
                {
                    Some(_message.data[index + self.bytes.len()..].to_vec())
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct MessageFilter {
    id: Option<Vec<u8>>,
    speed: Option<Speed>,
    filter_type: FilterType,
}

impl MessageFilter {
    pub(crate) fn new(id: Option<Vec<u8>>, speed: Option<Speed>, filter_type: FilterType) -> Self {
        Self {
            id,
            speed,
            filter_type,
        }
    }

    pub(crate) fn id(&self) -> Option<&Vec<u8>> {
        self.id.as_ref()
    }

    pub(crate) fn id_string(&self, ids: &Vec<HighlightID>) -> String {
        match self.id() {
            Some(id) => id_string(id, ids),
            None => String::from("any"),
        }
    }

    pub(crate) fn speed(&self) -> Option<&Speed> {
        self.speed.as_ref()
    }

    pub(crate) fn speed_string(&self) -> String {
        match self.speed() {
            Some(speed) => speed.to_string(),
            None => String::from("any"),
        }
    }

    pub(crate) fn filter_type(&self) -> &FilterType {
        &self.filter_type
    }

    pub(crate) fn description(&self) -> String {
        match self.filter_type() {
            FilterType::Basic => "Basic filter".to_string(),
            FilterType::StartsWithBytes(filter) => {
                format!("Data starts with {}", hex::encode(&filter.bytes))
            }
        }
    }

    pub(crate) fn output_data(&self, message: &Message) -> Option<Vec<u8>> {
        match self.filter_type() {
            FilterType::Basic => None,
            FilterType::StartsWithBytes(filter) => filter.output_data(message),
        }
    }

    pub(crate) fn filter(&self, message: &Message) -> bool {
        if let Some(id) = &self.id {
            if &message.id != id {
                return false;
            }
        }

        if let Some(speed) = &self.speed {
            if &message.speed != speed {
                return false;
            }
        }

        match &self.filter_type {
            FilterType::Basic => true,
            FilterType::StartsWithBytes(filter) => filter.filter_specific(message),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, EnumIter, Clone)]
pub enum FilterType {
    Basic,
    StartsWithBytes(StartsWithBytes),
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::Basic
    }
}

impl FilterType {
    pub(crate) fn is_variant(&self, variant: &FilterType) -> bool {
        match (self, variant) {
            (FilterType::StartsWithBytes(_), FilterType::StartsWithBytes(_)) => true,
            (FilterType::Basic, FilterType::Basic) => true,
            _ => false,
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            FilterType::StartsWithBytes(_) => "Starts with bytes",
            FilterType::Basic => "Basic",
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct LabelFilter {
    pub label: Label,
    pub filter: MessageFilter,
}

pub(crate) struct FilterResult {
    pub(crate) label: Label,
    pub(crate) output: Option<Vec<u8>>,
}
