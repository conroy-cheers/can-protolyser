use crate::filter::{FilterType, LabelFilter, MessageFilter};
use crate::gui::state::Field;
use crate::label::Label;
use crate::message::{id_string, HighlightID};

#[derive(Default)]
pub struct AddFilterLabelState {
    pub id: Field<String>,
    pub speed: Field<String>,
    pub filter_type: FilterType,
    pub name: Field<String>,
    pub color: Field<[f32; 3]>,
}

impl AddFilterLabelState {
    pub fn new() -> Self {
        Self {
            id: Field::default(),
            speed: Field::default(),
            filter_type: FilterType::default(),
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
        let filter_type = self.filter_type.clone();
        Some(LabelFilter {
            label: Label { name, color },
            filter: MessageFilter::new(Some(id), Some(speed), filter_type),
        })
    }

    pub fn id_text(&self, ids: &Vec<HighlightID>) -> String {
        match self.id.as_bytes(true) {
            None => self.id.value.clone(),
            Some(id) => id_string(&id, ids),
        }
    }

    pub fn speed_text(&self) -> String {
        match self.speed.as_string(true) {
            None => self.speed.value.clone(),
            Some(speed) => match speed.is_empty() {
                true => "any".to_string(),
                false => speed.clone(),
            },
        }
    }
}
