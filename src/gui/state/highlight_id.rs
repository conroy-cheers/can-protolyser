use eframe::egui::Color32;

use crate::{message::HighlightID, util::{hex_to_str, remove_whitespace}};

#[derive(Default)]
pub struct HighlightIDState {
    pub data: Vec<HighlightID>,
    editing_index: Option<usize>,
    pub edit_state: HighlightIDEditState,
}

impl HighlightIDState {
    pub(crate) fn from_data(data: Vec<HighlightID>) -> Self {
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
        self.edit_state = HighlightIDEditState::from_data(&self.data[index]);
    }

    pub(crate) fn commit(&mut self) {
        if let Some(index) = self.editing_index {
            if let Some(highlight_id) = self.edit_state.validate() {
                self.data[index] = highlight_id;
                self.editing_index = None;
                self.edit_state = HighlightIDEditState::default();
            }
        }
    }
}

pub struct HighlightIDEditState {
    pub id: String,
    pub id_valid: bool,
    pub name: String,
    pub name_valid: bool,
    pub color: [f32; 3],
}

impl Default for HighlightIDEditState {
    fn default() -> Self {
        Self {
            id: String::new(),
            id_valid: true,
            name: String::new(),
            name_valid: true,
            color: [255.0, 255.0, 255.0],
        }
    }
}

impl HighlightIDEditState {
    pub fn clear(&mut self) {
        self.id.clear();
        self.id_valid = true;
        self.name.clear();
        self.name_valid = true;
        self.color = [255.0, 255.0, 255.0];
    }

    pub(crate) fn from_data(data: &HighlightID) -> Self {
        Self {
            id: hex_to_str(data.id()),
            id_valid: true,
            name: data.name().clone(),
            name_valid: true,
            color: data.color().clone(),
        }
    }

    pub(crate) fn validate(&mut self) -> Option<HighlightID> {
        let id = self.validate_id()?;
        let name = self.validate_name()?;
        let color = self.color;

        Some(HighlightID::new(id, name, color))
    }

    pub fn validate_id(&mut self) -> Option<Vec<u8>> {
        let mut id = self.id.clone();
        remove_whitespace(&mut id);

        if id.len() == 0 {
            self.id_valid = false;
            return None;
        }
        if id.len() % 2 != 0 {
            self.id.insert(0, '0');
            return self.validate_id();
        }

        match hex::decode(&id) {
            Ok(id) => Some(id),
            Err(_) => {
                self.id_valid = false;
                return None;
            }
        }
    }

    pub fn validate_name(&mut self) -> Option<String> {
        if self.name.len() == 0 {
            self.name_valid = false;
            return None;
        }
        Some(self.name.clone())
    }

    pub fn clear_validation(&mut self) {
        self.id_valid = true;
        self.name_valid = true;
    }
}

impl HighlightID {
    pub fn color32(&self) -> Color32 {
        Color32::from_rgb(
            (self.color()[0] * 255.0) as u8,
            (self.color()[1] * 255.0) as u8,
            (self.color()[2] * 255.0) as u8,
        )
    }
}
