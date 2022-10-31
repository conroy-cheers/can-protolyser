use crate::filter::LabelFilter;
use crate::message::HighlightID;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Config {
    pub file_path: Option<PathBuf>,
    pub highlight_ids: Vec<HighlightID>,
    pub label_filters: Vec<LabelFilter>,
}

impl Config {
    pub fn new(file_path: Option<PathBuf>, highlight_ids: Vec<HighlightID>, filter_labels: Vec<LabelFilter>) -> Self {
        Self {
            file_path,
            highlight_ids,
            label_filters: filter_labels,
        }
    }
}

pub(crate) fn write_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create("config.json")?;
    serde_json::to_writer(&file, &config)?;
    Ok(())
}

pub(crate) fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file = fs::File::open("config.json")?;
    let config = serde_json::from_reader(&file)?;
    Ok(config)
}
