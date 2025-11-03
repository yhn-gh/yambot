use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSRequest {
    pub id: String,
    pub username: String,
    pub language: String,
    pub text: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct TTSAudioChunk {
    pub audio_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TTSQueueItem {
    pub request: TTSRequest,
    pub audio_chunks: Vec<TTSAudioChunk>,
}

#[derive(Debug, Clone)]
pub struct TTSQueue {
    queue: Arc<Mutex<VecDeque<TTSQueueItem>>>,
    ignored_users: Arc<Mutex<Vec<String>>>,
    currently_playing: Arc<Mutex<Option<TTSQueueItem>>>,
    skip_current: Arc<AtomicBool>,
}

impl TTSQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            ignored_users: Arc::new(Mutex::new(Vec::new())),
            currently_playing: Arc::new(Mutex::new(None)),
            skip_current: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn add(&self, item: TTSQueueItem) {
        let mut queue = self.queue.lock().await;
        queue.push_back(item);
    }

    pub async fn pop(&self) -> Option<TTSQueueItem> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }

    pub async fn peek(&self) -> Option<TTSQueueItem> {
        let queue = self.queue.lock().await;
        queue.front().cloned()
    }

    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        queue.clear();
    }

    pub async fn remove(&self, id: &str) -> bool {
        let mut queue = self.queue.lock().await;
        if let Some(pos) = queue.iter().position(|item| item.request.id == id) {
            queue.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn skip_current(&self) {
        self.skip_current.store(true, Ordering::SeqCst);
    }

    pub fn should_skip(&self) -> bool {
        self.skip_current.load(Ordering::SeqCst)
    }

    pub fn clear_skip(&self) {
        self.skip_current.store(false, Ordering::SeqCst);
    }

    pub fn get_skip_flag(&self) -> Arc<AtomicBool> {
        self.skip_current.clone()
    }

    pub async fn set_currently_playing(&self, item: Option<TTSQueueItem>) {
        let mut playing = self.currently_playing.lock().await;
        *playing = item;
    }

    pub async fn get_currently_playing(&self) -> Option<TTSQueueItem> {
        let playing = self.currently_playing.lock().await;
        playing.clone()
    }

    pub async fn get_all_with_current(&self) -> Vec<TTSQueueItem> {
        let mut result = Vec::new();

        // Add currently playing first
        if let Some(current) = self.get_currently_playing().await {
            result.push(current);
        }

        // Add queued items
        let queue = self.queue.lock().await;
        result.extend(queue.iter().cloned());

        result
    }

    pub async fn ignore_user(&self, username: &str) {
        let mut ignored = self.ignored_users.lock().await;
        if !ignored.contains(&username.to_string()) {
            ignored.push(username.to_string());
        }
    }

    pub async fn unignore_user(&self, username: &str) {
        let mut ignored = self.ignored_users.lock().await;
        ignored.retain(|u| u != username);
    }

    pub async fn is_user_ignored(&self, username: &str) -> bool {
        let ignored = self.ignored_users.lock().await;
        ignored.contains(&username.to_string())
    }

    pub async fn get_all(&self) -> Vec<TTSQueueItem> {
        let queue = self.queue.lock().await;
        queue.iter().cloned().collect()
    }

    pub async fn len(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.is_empty()
    }
}

impl Default for TTSQueue {
    fn default() -> Self {
        Self::new()
    }
}
