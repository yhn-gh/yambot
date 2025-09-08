use crate::backend::command::{Command, Parser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct TTSMessage {
    text: String,
    lang_code: String,
}

struct TTSHandler {
    queue: std::collections::VecDeque<TTSMessage>,
}

#[derive(Serialize, Deserialize)]
struct Language {
    lang_code: String,
    alias: String,
}

#[derive(Serialize, Deserialize)]
struct TTSConfig {
    langs: Vec<Language>,
    blocked_phrases: Vec<String>,
}

impl TTSMessage {
    pub fn new(text: String, lang_code: String) -> Self {
        Self { text, lang_code }
    }
}

impl Parser for TTSConfig {
    type Item = TTSMessage;

    fn parse(&self, c: &Command) -> Option<Self::Item> {
        let command = c.name();
        let args = c.args();
        let langs = &self.langs;
        if let Some(args) = args {
            let lang = langs.iter().find(|x| x.alias == command)?;
            let text = args.join(" ");

            // yucky clone; TODO: string interning
            Some(TTSMessage::new(text, lang.lang_code.to_owned()))
        } else {
            None
        }
    }
}
