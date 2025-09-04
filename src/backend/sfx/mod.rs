mod sounds;
mod tts;
mod watcher;
use serde::{Deserialize, Serialize};

use rodio::{OutputStream, OutputStreamBuilder};

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

use sounds::Soundlist;
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
#[non_exhaustive]
pub struct SoundsManager {
    sounds_path: PathBuf,
    watcher: Watcher,

    soundlist: Soundlist,
    ignore_list: HashSet<String>,

    stream_handle: OutputStream,
}

impl SoundsManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sounds_path = PathBuf::from(SOUNDS_DIRECTORY);

        let mut watcher = Watcher::serve();

        watcher.watch(&sounds_path)?;
        watcher.push_files()?;

        let soundlist = Soundlist::serve().await?;

        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        Ok(Self {
            sounds_path,
            watcher,
            soundlist,
            ignore_list: HashSet::new(),
            stream_handle,
        })
    }

    pub fn soundlist(&self) -> &Soundlist {
        &self.soundlist
    }

    pub fn watcher(&self) -> &Watcher {
        &self.watcher
    }

    pub fn get_stream_handle(self) -> OutputStream {
        self.stream_handle
    }
}
