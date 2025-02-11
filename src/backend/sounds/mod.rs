pub mod sfx;
pub mod watcher;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

static IGNORELIST_PATH: &str = "./assets/ignore_list.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IgnoreList {
    ignored_users: HashSet<String>,
}

impl IgnoreList {
    fn new() -> Result<IgnoreList, Box<dyn std::error::Error>> {
        let ignore_list = Path::new(IGNORELIST_PATH);

        if !ignore_list.is_file() {
            let file = File::create(IGNORELIST_PATH)?;
            serde_json::to_writer(file, &IgnoreList::default())?;
        };

        let file = File::open(IGNORELIST_PATH)?;

        let reader = BufReader::new(file);
        let ignore_list: IgnoreList = serde_json::from_reader(reader)?;
        Ok(ignore_list)
    }
}
//TODO implement automatic gain control + ignore users both frontend and backend
