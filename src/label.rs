use crate::egui::Color32;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Label {
    pub name: String,
    pub color: [f32; 3],
}

impl Label {
    pub(crate) fn color32(&self) -> Color32 {
        Color32::from_rgb(
            (self.color[0] * 255.0) as u8,
            (self.color[1] * 255.0) as u8,
            (self.color[2] * 255.0) as u8,
        )
    }
}
