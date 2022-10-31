use crate::util::remove_whitespace;
use serde::{Deserialize, Serialize};

fn hex_deserializer<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    s = s.to_lowercase();
    remove_whitespace(&mut s);
    s = s.strip_prefix("0x").unwrap().to_string();

    if s.len() % 2 != 0 {
        s.insert(0, '0');
    }
    hex::decode(s).map_err(serde::de::Error::custom)
}

fn true_false_deserializer<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.to_lowercase();
    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom("Invalid boolean")),
    }
}

pub type Speed = String;

#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    pub timestamp: f64,
    #[serde(deserialize_with = "hex_deserializer")]
    pub id: Vec<u8>,
    #[serde(deserialize_with = "hex_deserializer")]
    pub data: Vec<u8>,
    #[serde(deserialize_with = "hex_deserializer")]
    pub crc: Vec<u8>,
    #[serde(deserialize_with = "true_false_deserializer")]
    pub ack: bool,
    pub speed: Speed,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct MyHex {
    #[serde(with = "hex::serde")]
    #[allow(dead_code)] // Used for transparent deserialize
    hex: Vec<u8>,
}

impl Message {
    pub fn match_id(&self, highlight_ids: &Vec<HighlightID>) -> Option<HighlightID> {
        for id in highlight_ids {
            if self.id == id.id {
                return Some(id.clone());
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HighlightID {
    id: Vec<u8>,
    name: String,
    color: [f32; 3],
}

impl HighlightID {
    pub(crate) fn new(id: Vec<u8>, name: String, color: [f32; 3]) -> Self {
        Self { id, name, color }
    }

    pub(crate) fn id(&self) -> &Vec<u8> {
        &self.id
    }

    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    pub(crate) fn color(&self) -> &[f32; 3] {
        &self.color
    }
}

pub(crate) fn id_string(id: &Vec<u8>, ids: &Vec<HighlightID>) -> String {
    match id.is_empty() {
        true => "any".to_string(),
        false => match ids.iter().find(|h_id| h_id.id == *id) {
            None => hex::encode(id),
            Some(h_id) => h_id.name.clone(),
        },
    }
}
