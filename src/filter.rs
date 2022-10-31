use strum::EnumIter;

use crate::label::Label;
use crate::message::{HighlightID, Message, Speed, id_string};

pub trait SpecialFilter {
    fn filter_specific(&self, message: &Message) -> bool;
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct StartsWithBytes {
    bytes: Vec<u8>,
}

impl SpecialFilter for StartsWithBytes {
    fn filter_specific(&self, message: &Message) -> bool {
        message.data.starts_with(&self.bytes)
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
            FilterType::StartsWithBytes(filter) => message.data.starts_with(&filter.bytes),
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
    // pub(crate) fn message_filter(&self) -> &dyn SpecialFilter {
    //     match self {
    //         FilterType::StartsWithBytes(filter) => filter,
    //         FilterType::Basic(filter) => filter,
    //     }
    // }

    // pub(crate) fn mut_message_filter(&mut self) -> &mut dyn SpecialFilter {
    //     match self {
    //         FilterType::StartsWithBytes(filter) => filter,
    //         FilterType::Basic(filter) => filter,
    //     }
    // }

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
