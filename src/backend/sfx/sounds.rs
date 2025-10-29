use crate::backend::command::{Command, Parser};
use rodio::{OutputStream, OutputStreamHandle};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path, path::PathBuf, sync::Arc};

use super::{Format, FILES};
use crate::backend::config;

const SOUNDLIST_PATH: &str = "./assets/soundlist.json";
const SOUNDS_DIRECTORY: &str = "./assets/sounds/";

// type StreamHandle = Arc<OutputStreamHandle>;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
// rename it to SoundConfig or SfxConfig, if its gonna have more than just hashset
pub struct Soundlist {
    sounds: HashSet<String>,
}

type Sound = Arc<Path>;

impl Parser<'_> for Soundlist {
    type Item = Sound;
    fn parse(&self, c: &Command) -> Option<Self::Item> {
        let command = c.name();
        if self.sounds.contains(command) {
            let path = Path::new("{SOUNDLIST_PATH}{sound}");
            let sound = Arc::from(path);
            Some(sound)
        } else {
            None
        }
    }
}

impl Soundlist {
    pub async fn serve() -> Result<Self, Box<dyn std::error::Error>> {
        if !tokio::fs::try_exists(SOUNDLIST_PATH).await? {
            let default = serde_json::to_vec(&Self::default())?;
            tokio::fs::write(SOUNDLIST_PATH, default).await?;
        };

        let soundlist_json = tokio::fs::read(SOUNDLIST_PATH).await?;
        let mut sounds: Self = serde_json::from_reader(&*soundlist_json)?;

        sounds.sync_files()?;

        sounds.save().await?;
        Ok(sounds)
    }

    pub fn is_soundfile(file: &PathBuf) -> Option<(&str, &str)> {
        let sound_format = Self::get_format();
        match (file.file_stem(), file.extension()) {
            (Some(filename), Some(extension)) if extension == sound_format => {
                Some((filename.to_str()?, extension.to_str()?))
            }
            _ => None,
        }
    }

    pub fn get_format<'a>() -> &'a str {
        let sound_format: &str = match config::load_config().chatbot.sound_format {
            Format::Wav => "wav",
            Format::Opus => "opus",
            Format::Mp3 => "mp3",
        };
        sound_format
    }

    fn sync_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut lock = FILES.lock()?;
        match std::fs::read_dir(SOUNDS_DIRECTORY) {
            Ok(entries) => {
                self.sounds.clear();
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let file = entry.path();
                            if let Some((filename, _)) = Self::is_soundfile(&file) {
                                self.sounds.insert(filename.to_owned());
                                lock.insert(filename.to_owned());
                            }
                        }
                        Err(e) => log::error!("Sound file error: {}", e),
                    }
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Sound directory error: {}", e);
                Err(Box::new(e))
            }
        }
    }

    async fn save(&self) -> Result<(), std::io::Error> {
        let sounds = serde_json::to_vec(self)?;
        tokio::fs::write(SOUNDLIST_PATH, &sounds).await?;
        Ok(())
    }
}
