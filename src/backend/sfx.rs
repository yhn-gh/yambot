
use super::config::AppConfig; 

use serde::{Serialize, Deserialize};
use serde_json::json;

use rodio::{Decoder, OutputStreamHandle, Sink};
use std::io::{BufReader, BufWriter};
use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::sync::Arc;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct SoundList {
    path: String,
    files: HashSet<String>,
}

impl SoundList {
    pub fn new(p: &Path) -> Result<Self, Box<dyn Error>> {
        let files: HashSet<String> = HashSet::new();
        let path: String = String::from(p.to_str().unwrap());
        if !Path::exists(p) {
            let writer = BufWriter::new(fs::File::create(p)?);
            let v = json!(Self {path: path.clone(), files: files.clone()});
            serde_json::to_writer(writer, &v)?;
        } 

        Ok(Self {path, files})
    }

    pub fn add(&mut self, file: &Path) -> Result<(), Box<dyn Error>> {

        let file_name = String::from(
            file
            .file_name()
            .expect("Sound file doesn't exist or isn't a file.")
            .to_str()
            .unwrap()
        );
        log::info!("Pre: {:?}", self.files);
        self.files.insert(String::from(file_name));
        log::info!("Post: {:?}", self.files);
        let writer = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        serde_json::to_writer(&writer, &self)?;

        Ok(())
    }

    fn find(&mut self, file: &Path) -> Option<()> {

        Some(())
    }
}

pub struct Playable;

impl Playable {
    async fn play(file: fs::File, stream_handle: Arc<OutputStreamHandle>) -> Result<(), Box<dyn Error>> {
        let source = Decoder::new(BufReader::new(file))?;
        let sink = Sink::try_new(&stream_handle)?;

        sink.set_volume(1.0);
        sink.append(source);
        sink.detach();

        Ok(())
    }
}
