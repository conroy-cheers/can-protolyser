use std::error::Error;
use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::Sender;
use std::thread;

use crate::message::Message;

fn parse_file(path: &PathBuf, progress: &Mutex<f32>) -> Result<Vec<Message>, Box<dyn Error>> {
    let reader = csv::Reader::from_path(path)?;
    let num_lines = reader.into_records().count();

    let mut msgs = Vec::with_capacity(num_lines);
    let mut lines_read = 0;

    let mut reader = csv::Reader::from_path(path)?;
    for result in reader.deserialize() {
        lines_read += 1;
        let progress_val=  lines_read as f32 / num_lines as f32;
        *progress.lock().unwrap() = progress_val;
        msgs.push(result?);
    }
    Ok(msgs)
}

pub fn load_file_async(path: &PathBuf, progress: Arc<Mutex<f32>>, result: Sender<Option<Vec<Message>>>) {
    let path = path.clone();
    thread::spawn(move || {
        let msgs = parse_file(&path, progress.as_ref());
        match msgs {
            Ok(msgs) => {
                match result.send(Some(msgs)) {
                    Ok(_) => (),
                    Err(_) => return,
                }
            }
            Err(_e) => {
                result.send(None).unwrap();
            }
        }
    });
}
