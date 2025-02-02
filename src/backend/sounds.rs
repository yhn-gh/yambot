use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{BufReader, Write},
    path::Path,
    sync::{LazyLock, Mutex},
};

static SOUNDLIST_PATH: &str = "./assets/soundlist.json";
static SOUNDS_DIRECTORY: &str = "./assets/sounds/";
pub static FILES: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum Format {
    Wav,
    Opus,
    Mp3,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Sounds {
    files: HashSet<String>,
}

impl Sounds {
    pub fn serve() -> Result<(), Box<dyn std::error::Error>> {
        let soundlist = Path::new(SOUNDLIST_PATH);

        if !soundlist.is_file() {
            let mut file = File::create(SOUNDLIST_PATH)?;
            write!(file, "{}", serde_json::to_string(&Self::default())?)?;
        };
        let file = File::open(SOUNDLIST_PATH)?;

        let reader = BufReader::new(file);
        let mut sounds: Self = serde_json::from_reader(reader)?;
        let sync = sounds.sync_files();
        if sync.is_ok() {
            let files = &mut sounds.files;
            let mut lock = FILES.lock()?;
            files.iter().for_each(|x| {
                lock.insert(x.to_string());
            });
        }

        sounds.save()?;
        Ok(())
    }

    pub fn get_format() -> String {
        let sound_format: &str = match super::config::load_config().chatbot.sound_format {
            Format::Wav => "wav",
            Format::Opus => "opus",
            Format::Mp3 => "mp3",
        };
        sound_format.to_string()
    }

    fn sync_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match fs::read_dir(SOUNDS_DIRECTORY) {
            Ok(entries) => {
                self.files.clear();
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let file = entry.file_name().into_string().unwrap();

                            let mut file_name = file.rsplitn(2, '.');
                            let extension = file_name.next().unwrap_or("");
                            let file = file_name.next().unwrap_or(&file);
                            let sound_format = Self::get_format();

                            if extension == sound_format {
                                self.files.insert(String::from(file));
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

    fn save(&mut self) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(SOUNDLIST_PATH)
            .expect("Failed to open the file");

        let files = FILES.lock().unwrap().clone();
        self.files = files;

        write!(file, "{}", json!(self))?;

        Ok(())
    }
}
