use std::path::PathBuf;

use native_dialog::FileDialog;

pub fn csv_from_dialog() -> Option<PathBuf> {
    match FileDialog::new()
        .add_filter("CSV", &["csv"])
        .show_open_single_file()
        .unwrap()
    {
        Some(path) => Some(path),
        None => None,
    }
}
