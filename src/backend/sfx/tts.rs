use crate::backend::command::{Command, Parser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct TTSMessage<'a> {
    text: String,
    language: &'a Language,
}

struct TTSHandler<'a> {
    queue: std::collections::VecDeque<TTSMessage<'a>>,
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

impl<'a> TTSMessage<'a> {
    pub fn new(text: String, language: &'a Language) -> Self {
        Self { text, language }
    }
    
    pub async fn recv_tts(&self) -> Result<reqwest::Response, Box< dyn std::error::Error>>{
        let recv = reqwest::get("https://translate.google.com/translate_tts?ie=UTF-8&tl={self.lang_code}&client=tw-ob&q={self.text}").await?;
        Ok(recv)
    }
}

impl<'p> Parser<'p> for TTSConfig {
    type Item = TTSMessage<'p>;

    fn parse(&'p self, c: &Command) -> Option<Self::Item> {
        let command = c.name();
        let args = c.args();
        let langs = &self.langs;
        if let Some(args) = args {
            let lang = langs.iter().find(|x| x.alias == command)?;
            let text = args.join(" ");
            
            Some(TTSMessage::new(text, lang))
        } else {
            None
        }
    }
}
