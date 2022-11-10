use crate::filter::{
    FilterResult, FilterType, LabelFilter, MessageFilter, OutputSelection, StartsWithBytes,
};
use crate::gui::state::{Field, ParseError};
use crate::label::Label;
use crate::message::Message;
use crate::util::{empty_str_as_none, empty_vec_as_none, hex_to_str};

#[derive(Default)]
pub(crate) struct FilterLabelState {
    pub(crate) data: Vec<LabelFilter>,
    editing_index: Option<usize>,
    pub(crate) edit_state: EditFilterLabelState,
}

impl FilterLabelState {
    pub(crate) fn from_data(data: Vec<LabelFilter>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    pub(crate) fn editing_index(&self) -> &Option<usize> {
        &self.editing_index
    }

    pub(crate) fn edit(&mut self, index: usize) {
        self.editing_index = Some(index);
        self.edit_state = EditFilterLabelState::from_data(&self.data[index]);
    }

    pub(crate) fn commit(&mut self) {
        if let Some(index) = self.editing_index {
            if let Ok(highlight_id) = self.edit_state.validate() {
                self.data[index] = highlight_id;
                self.editing_index = None;
                self.edit_state = EditFilterLabelState::default();
            }
        }
    }

    pub(crate) fn matching_labels(&self, message: &Message) -> Vec<FilterResult> {
        self.data
            .iter()
            .filter(|lf| lf.filter.filter(message))
            .map(|lf| FilterResult {
                label: lf.label.clone(),
                output: lf.filter.output_data(message),
            })
            .collect()
    }
}

pub(crate) struct EditFilterLabelState {
    pub id: Field<String>,
    pub speed: Field<String>,
    pub filter_type: FilterType,
    pub filter_options: EditFilterOptionsState,
    pub name: Field<String>,
    pub color: Field<[f32; 3]>,
}

impl Default for EditFilterLabelState {
    fn default() -> Self {
        Self {
            id: Field::default(),
            speed: Field::default(),
            filter_type: FilterType::default(),
            filter_options: EditFilterOptionsState::default(),
            name: Field::default(),
            color: Field::<[f32; 3]> {
                value: [255.0, 255.0, 255.0],
                valid: true,
            },
        }
    }
}

impl EditFilterLabelState {
    pub(crate) fn from_data(data: &LabelFilter) -> Self {
        Self {
            id: Field::with_value(data.filter.id().map_or("".to_string(), |id| hex_to_str(id))),
            speed: Field::with_value(
                data.filter
                    .speed()
                    .map_or("".to_string(), |speed| speed.to_string()),
            ),
            filter_type: data.filter.filter_type().clone(),
            filter_options: EditFilterOptionsState::from_filter_type(data.filter.filter_type()),
            name: Field::with_value(data.label.name.clone()),
            color: Field::with_value(data.label.color),
        }
    }

    pub(crate) fn validate(&mut self) -> Result<LabelFilter, ParseError> {
        let id = empty_vec_as_none(self.id.validate_bytes(true)?);
        let speed = empty_str_as_none(self.speed.validate_string(true)?);
        let name = self.name.validate_string(false)?;
        let color = self.color.value;
        let filter_type = match (&self.filter_type, &mut self.filter_options) {
            (FilterType::Basic, EditFilterOptionsState::Empty) => FilterType::Basic,
            (
                FilterType::StartsWithBytes(_),
                EditFilterOptionsState::OneStringFieldOneOutputSelection(field, output),
            ) => FilterType::StartsWithBytes(StartsWithBytes {
                bytes: field.validate_bytes(false)?,
                output: output.clone(),
            }),
            _ => return Err(ParseError {}),
        };
        Ok(LabelFilter {
            label: Label { name, color },
            filter: MessageFilter::new(id, speed, filter_type),
        })
    }
}

#[derive(Default)]
pub(crate) enum EditFilterOptionsState {
    #[default]
    Empty,
    OneStringField(Field<String>),
    OneStringFieldOneOutputSelection(Field<String>, OutputSelection),
}

impl EditFilterOptionsState {
    pub(crate) fn from_filter_type(filter_type: &FilterType) -> Self {
        match filter_type {
            FilterType::Basic => Self::Empty,
            FilterType::StartsWithBytes(StartsWithBytes { bytes, output }) => {
                Self::OneStringFieldOneOutputSelection(
                    Field::with_value(hex_to_str(bytes)),
                    output.clone(),
                )
            }
        }
    }
}
