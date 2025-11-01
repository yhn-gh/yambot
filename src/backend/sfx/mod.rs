mod sounds;
mod watcher;
use serde::{Deserialize, Serialize};

use rodio::OutputStream;

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

pub use sounds::Soundlist;
use watcher::Watcher;

// MAYBE ADD AN OPTION TO CHANGE THE DIRECTORY TO A DIFFERENT ONE IN CONFIG
static SOUNDS_DIRECTORY: &str = "./assets/sounds/";

pub static FILES: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum Format {
    Wav,
    Opus,
    Mp3,
}

// public interface for all things sounds/sfx-related
pub struct SoundsManager {
    #[allow(dead_code)] // Reserved for future sound directory management
    sounds_path: PathBuf,
    watcher: Watcher,

    soundlist: Soundlist,
    #[allow(dead_code)] // Reserved for filtering unwanted sounds
    ignore_list: HashSet<String>,

    stream: OutputStream,
}

impl SoundsManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sounds_path = PathBuf::from(SOUNDS_DIRECTORY);

        let mut watcher = Watcher::serve();

        watcher.watch(&sounds_path)?;
        watcher.push_files()?;

        let soundlist = Soundlist::serve().await?;

        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Failed to open default audio stream");

        Ok(Self {
            sounds_path,
            watcher,
            soundlist,
            ignore_list: HashSet::new(),
            stream,
        })
    }

    pub fn soundlist(&self) -> &Soundlist {
        &self.soundlist
    }

    pub fn watcher(&self) -> &Watcher {
        &self.watcher
    }

    pub fn get_stream(self) -> OutputStream {
        self.stream
    }
}
