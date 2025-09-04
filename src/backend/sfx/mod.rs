mod sounds;
mod watcher;
use serde::{Deserialize, Serialize};

use rodio::{OutputStream, OutputStreamHandle};

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

#[tokio::test]
async fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
    use crate::backend::Command;
    use crate::backend::Parser;
    let soundlist = Soundlist::serve().await?;

    let cmd = Command {
        name: "test".into(),
        args: None,
    };
    assert!(
        soundlist.parse(&cmd).is_some(),
        "No sound file named `test`"
    );
    Ok(())
}

// public interface for all things sounds/sfx-related
#[non_exhaustive]
pub struct SoundsManager {
    sounds_path: PathBuf,
    watcher: Watcher,

    soundlist: Soundlist,
    ignore_list: HashSet<String>,

    stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl SoundsManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sounds_path = PathBuf::from(SOUNDS_DIRECTORY);

        let mut watcher = Watcher::serve();

        watcher.watch(&sounds_path)?;
        watcher.push_files()?;

        let soundlist = Soundlist::serve().await?;

        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            sounds_path,
            watcher,
            soundlist,
            ignore_list: HashSet::new(),
            stream,
            stream_handle,
        })
    }

    pub fn soundlist(&self) -> &Soundlist {
        &self.soundlist
    }

    pub fn watcher(&self) -> &Watcher {
        &self.watcher
    }

    pub fn get_stream(self) -> (OutputStream, OutputStreamHandle) {
        (self.stream, self.stream_handle)
    }
}
