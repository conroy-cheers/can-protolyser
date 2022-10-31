use eframe::egui::Color32;

use crate::message::HighlightID;

pub struct AddHighlightIDState {
    pub id: String,
    pub id_valid: bool,
    pub name: String,
    pub name_valid: bool,
    pub color: [f32; 3],
}

impl AddHighlightIDState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            id_valid: true,
            name: String::new(),
            name_valid: true,
            color: [255.0, 255.0, 255.0],
        }
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.id_valid = true;
        self.name.clear();
        self.name_valid = true;
        self.color = [255.0, 255.0, 255.0];
    }

    pub fn validate_id(&mut self) -> Option<Vec<u8>> {
        if self.id.len() == 0 {
            self.id_valid = false;
            return None;
        }
        if self.id.len() % 2 != 0 {
            self.id.insert(0, '0');
        }

        match hex::decode(&self.id) {
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
