use crate::backend::config;
use crate::{ChatMessage, Roles};
use rodio::{Decoder, OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{BufReader, Write},
    path::Path,
    sync::{Arc, LazyLock, Mutex},
};
use tokio::sync::mpsc;

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
struct Soundlist {
    files: HashSet<String>,
}

pub struct Sounds {
    tx: mpsc::Sender<ChatMessage>,
    ignore_list: super::IgnoreList,
}

impl Sounds {
    pub fn serve(stream_handle: Arc<OutputStreamHandle>) -> Self {
        let (in_tx, mut in_rx) = mpsc::channel(100);
        let (out_tx, mut out_rx) = mpsc::channel(100);

        Self::to_files().expect("Couldn't sync files");
        tokio::spawn(Self::fan_in(in_rx, out_tx));
        tokio::spawn(Self::fan_out(out_rx, stream_handle));

        Self {
            tx: in_tx,
            ignore_list: super::IgnoreList::new().unwrap(),
        }
    }
    pub fn to_files() -> Result<(), Box<dyn std::error::Error>> {
        let soundlist = Path::new(SOUNDLIST_PATH);
        if !soundlist.is_file() {
            let file = File::create(SOUNDLIST_PATH)?;
            serde_json::to_writer(file, &Soundlist::default())?;
        };

        let sounds = File::open(SOUNDLIST_PATH)?;
        let reader = BufReader::new(sounds);
        let mut sounds: Soundlist = serde_json::from_reader(reader)?;
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

    pub async fn fan_out(mut rx: mpsc::Receiver<String>, stream_handle: Arc<OutputStreamHandle>) {
        while let Some(rx) = rx.recv().await {}
    }

    pub async fn fan_in(mut rx: mpsc::Receiver<ChatMessage>, tx: mpsc::Sender<String>) {
        while let Some(message) = rx.recv().await {
            let command = message.message_text.split_whitespace().next().unwrap();
            let config = config::load_config().sfx;
            // this is ugly as fuck hope this irc shit is gone for good
            let temp_role_closure_for_irc = |x: &str, lvl: &str| match x {
                "broadcaster" => Roles::Broadcaster,
                "moderator" => Roles::Moderator,
                "vip" => Roles::Vip,
                "sub" => Roles::Sub(lvl.trim().parse::<usize>().unwrap()),
                _ => Roles::None,
            };
            let badges: Vec<Roles> = message
                .badges
                .iter()
                .map(|x| {
                    x.split_once('-')
                        .map(|(a, b)| temp_role_closure_for_irc(a, b))
                        .unwrap()
                })
                .collect();
            let permited: bool = badges.iter().any(|badge| match badge {
                Roles::Broadcaster => true,
                Roles::Moderator => true,
                Roles::Vip => config.permited_roles.vips,
                Roles::Sub(_) => config.permited_roles.subs,
                // this should be an option maybe probably
                Roles::None => false,
            });
            let contains = FILES.lock().unwrap().contains(command);
            if permited && contains {
                let file = format!("{}.{}", command, Sounds::get_format());
                let _ = tx.send(file).await;
            }
        }
    }

    pub fn get_format() -> String {
        match config::load_config().chatbot.sound_format {
            Format::Wav => "wav",
            Format::Opus => "opus",
            Format::Mp3 => "mp3",
        }
        .into()
    }
    async fn play_sound(sound_file: &str, stream_handle: Arc<OutputStreamHandle>) {
        let sound_path = "./assets/sounds/".to_string() + sound_file;
        if let Ok(file) = File::open(Path::new(&sound_path)) {
            let source = Decoder::new(BufReader::new(file)).unwrap();
            let sink = Sink::try_new(&*stream_handle).unwrap();
            sink.set_volume(0.5);
            sink.append(source);
            sink.detach();
        } else {
            println!("Could not open sound file: {}", sound_path);
        }
    }

    pub fn get_filename(string: String) -> Option<String> {
        // bit garbage-y but it works
        let path = Path::new(&string);
        path.extension()
            .and_then(|x| x.to_str())
            .filter(|x| *x == Self::get_format())
            .and_then(|_| path.file_stem())
            .map(|_| path.to_string_lossy().into_owned())
    }
}

impl Soundlist {
    fn sync_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match fs::read_dir(SOUNDS_DIRECTORY) {
            Ok(entries) => {
                self.files.clear();
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let file = entry.file_name().into_string().unwrap();
                            if let Some(filename) = Sounds::get_filename(file) {
                                self.files.insert(filename);
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
