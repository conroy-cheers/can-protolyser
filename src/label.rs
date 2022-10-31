#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Label {
    pub name: String,
    pub color: [f32; 3],
}
