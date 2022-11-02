use crate::filter::{FilterType, LabelFilter, MessageFilter, StartsWithBytes};
use crate::gui::state::Field;
use crate::label::Label;
use crate::util::hex_to_str;

#[derive(Default)]
pub(crate) struct FilterLabelEditState {
    pub id: Field<String>,
    pub speed: Field<String>,
    pub filter_type: FilterType,
    pub filter_options: AddFilterOptionsState,
    pub name: Field<String>,
    pub color: Field<[f32; 3]>,
}

impl FilterLabelEditState {
    pub fn new() -> Self {
        Self {
            id: Field::default(),
            speed: Field::default(),
            filter_type: FilterType::default(),
            filter_options: AddFilterOptionsState::default(),
            name: Field::default(),
            color: Field::<[f32; 3]> {
                value: [255.0, 255.0, 255.0],
                valid: true,
            },
        }
    }

    pub(crate) fn validate(&mut self) -> Option<LabelFilter> {
        let id = self.id.validate_bytes(true)?;
        let speed = self.speed.validate_string(true)?;
        let name = self.name.validate_string(false)?;
        let color = self.color.value;
        let filter_type = match (&self.filter_type, &mut self.filter_options) {
            (FilterType::Basic, AddFilterOptionsState::Empty) => FilterType::Basic,
            (FilterType::StartsWithBytes(_), AddFilterOptionsState::OneStringField(ref mut field)) => {
                FilterType::StartsWithBytes(StartsWithBytes {
                    bytes: field.validate_bytes(false)?,
                })
            }
            _ => return None,
        };
        Some(LabelFilter {
            label: Label { name, color },
            filter: MessageFilter::new(Some(id), Some(speed), filter_type),
        })
    }
}

#[derive(Default)]
pub(crate) enum AddFilterOptionsState {
    #[default]
    Empty,
    OneStringField(Field<String>),
}

impl AddFilterOptionsState {
    pub(crate) fn from_filter_type(filter_type: &FilterType) -> Self {
        match filter_type {
            FilterType::Basic => Self::Empty,
            FilterType::StartsWithBytes(StartsWithBytes { bytes }) => {
                Self::OneStringField(Field::with_value(hex_to_str(bytes)))
            }
        }
    }
}
