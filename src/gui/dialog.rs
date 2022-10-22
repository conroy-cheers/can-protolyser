use native_dialog::{Error, FileDialog};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DialogError {
    err: Error,
    msg: Option<String>,
}

impl fmt::Display for DialogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.msg {
            Some(ref msg) => write!(f, "{}: {}", self.err, msg),
            None => write!(f, "{}", self.err),
        }
    }
}

pub fn csv_from_dialog() -> Result<Option<PathBuf>, DialogError> {
    match FileDialog::new()
        .add_filter("CSV", &["csv"])
        .show_open_single_file()
    {
        Ok(Some(path)) => Ok(Some(path)),
        Ok(None) => Ok(None),
        Err(Error::ImplementationError(msg)) => Err(DialogError {
            err: Error::ImplementationError(msg.clone()),
            msg: Some(msg),
        }),
        Err(e) => Err(DialogError { err: e, msg: None }),
    }
}
