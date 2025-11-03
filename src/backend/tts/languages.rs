use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub languages: HashMap<String, Language>,
}

impl LanguageConfig {
    pub fn new() -> Self {
        Self {
            languages: HashMap::new(),
        }
    }

    pub fn get_language(&self, code: &str) -> Option<&Language> {
        self.languages.get(code)
    }

    pub fn is_enabled(&self, code: &str) -> bool {
        self.languages
            .get(code)
            .map(|lang| lang.enabled)
            .unwrap_or(false)
    }

    pub fn toggle_language(&mut self, code: &str) {
        if let Some(lang) = self.languages.get_mut(code) {
            lang.enabled = !lang.enabled;
        }
    }

    pub fn enable_language(&mut self, code: &str) {
        if let Some(lang) = self.languages.get_mut(code) {
            lang.enabled = true;
        }
    }

    pub fn disable_language(&mut self, code: &str) {
        if let Some(lang) = self.languages.get_mut(code) {
            lang.enabled = false;
        }
    }

    pub fn get_enabled_languages(&self) -> Vec<&Language> {
        self.languages
            .values()
            .filter(|lang| lang.enabled)
            .collect()
    }

    pub fn get_all_languages(&self) -> Vec<&Language> {
        let mut langs: Vec<&Language> = self.languages.values().collect();
        langs.sort_by(|a, b| a.name.cmp(&b.name));
        langs
    }
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self::new()
    }
}
