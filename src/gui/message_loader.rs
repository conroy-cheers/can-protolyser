use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use crate::file::load_file_async;
use crate::message::{Message, Speed};

pub struct MessageLoader {
    state: MessageLoaderState,
    known_speeds: HashSet<Speed>
}

pub enum MessageLoaderState {
    FileNotSelected,
    FileSelected(PathBuf),
    Loading {
        progress: Arc<Mutex<f32>>,
        file_path: PathBuf,
        loader_channel: (Sender<Option<Vec<Message>>>, Receiver<Option<Vec<Message>>>),
    },
    Loaded {
        messages: Vec<Message>,
        file_path: PathBuf,
    },
    Error {
        file_path: Option<PathBuf>,
        error: Option<String>,
    },
}

impl MessageLoaderState {
    fn file_path(&self) -> Option<&PathBuf> {
        match self {
            MessageLoaderState::FileNotSelected => None,
            MessageLoaderState::FileSelected(file_path) => Some(file_path),
            MessageLoaderState::Loading { file_path, .. } => Some(file_path),
            MessageLoaderState::Loaded { file_path, .. } => Some(file_path),
            MessageLoaderState::Error { file_path, .. } => file_path.as_ref(),
        }
    }
}

impl MessageLoader {
    pub fn new() -> Self {
        Self {
            state: MessageLoaderState::FileNotSelected,
            known_speeds: HashSet::new(),
        }
    }

    pub fn state(&self) -> &MessageLoaderState {
        &self.state
    }

    pub fn from_path(file_path: PathBuf) -> Self {
        Self {
            state: MessageLoaderState::FileSelected(file_path),
            known_speeds: HashSet::new(),
        }
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        self.state.file_path()
    }

    pub fn replace_file_path(&mut self, file_path: Option<PathBuf>) {
        match file_path {
            None => self.state = MessageLoaderState::FileNotSelected,
            Some(path) => self.state = MessageLoaderState::FileSelected(path),
        }
    }

    pub fn loading_progress(&self) -> f32 {
        match &self.state {
            MessageLoaderState::Loading { progress, .. } => progress.lock().unwrap().clone(),
            MessageLoaderState::Loaded { .. } => 1.0,
            _ => 0.0,
        }
    }

    pub fn known_speeds(&self) -> &HashSet<Speed> {
        &self.known_speeds
    }

    pub fn set_error(&mut self, error: String) {
        self.state = MessageLoaderState::Error {
            file_path: self.state.file_path().cloned(),
            error: Some(error),
        }
    }

    pub fn handle_file_loading(&mut self) {
        match &self.state {
            MessageLoaderState::Error { .. } => (),
            MessageLoaderState::FileNotSelected => (),
            MessageLoaderState::Loaded { .. } => (),
            MessageLoaderState::FileSelected(file_path) => {
                // Start loading the file
                let (sender, receiver) = std::sync::mpsc::channel();
                let progress = Arc::new(Mutex::new(0.0));
                load_file_async(&file_path, progress.clone(), sender.clone());
                self.state = MessageLoaderState::Loading {
                    progress,
                    file_path: file_path.clone(),
                    loader_channel: (sender, receiver),
                };
            }
            MessageLoaderState::Loading {
                file_path,
                loader_channel,
                ..
            } => {
                // Update progress
                let (_sender, receiver) = loader_channel;
                match receiver.try_recv() {
                    Err(TryRecvError::Empty) => {} // Still loading
                    Err(TryRecvError::Disconnected) => {
                        // Loader thread died
                        self.state = MessageLoaderState::Error {
                            file_path: Some(file_path.clone()),
                            error: Some("File load ended unexpectedly".to_string()),
                        };
                    }
                    Ok(Some(messages)) => {
                        // Load succeeded
                        self.known_speeds = messages.iter().map(|m| m.speed.clone()).collect();
                        self.state = MessageLoaderState::Loaded {
                            messages,
                            file_path: file_path.clone(),
                        };
                    }
                    Ok(None) => {
                        // Load failed, presumably due to invalid file contents
                        self.state = MessageLoaderState::Error {
                            file_path: Some(file_path.clone()),
                            error: Some("File invalid".to_string()),
                        };
                    }
                }
            }
        }
    }
}
