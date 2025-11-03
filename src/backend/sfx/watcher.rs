use notify::{
    event::{EventKind, ModifyKind, RenameMode},
    RecommendedWatcher, Watcher as _Watcher,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    pin,
    sync::{mpsc, watch},
};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

pub struct Watcher {
    in_tx: watch::Sender<PathBuf>,
    events: watch::Receiver<HashSet<SoundEvent>>,
}

use super::Soundlist;
use super::FILES;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SoundEvent {
    Add(PathBuf),
    Remove(PathBuf),
}

impl Watcher {
    pub fn serve() -> Self {
        // notify channel
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        // watching directories channel
        let (in_tx, in_rx) = watch::channel(PathBuf::new());
        // fan-out notify event chunks -> push_files loop
        let (to_fan, events) = watch::channel(HashSet::new());

        let handler = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                let Ok(mut event) = res else {
                    log::error!("Watcher Error: {:?}", res);
                    return;
                };

                // pop removes first parent
                let file = event.paths.pop().expect("No Such file exists");

                log::debug!("File watcher event: {:?} for file: {}", event.kind, file.display());

                // Check if file exists to determine if it's a creation or deletion
                let file_exists = file.exists();

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                        log::info!("Detected sound file creation: {}", file.display());
                        let event = SoundEvent::Add(file);
                        out_tx.send(event).unwrap();
                    }
                    EventKind::Remove(_)
                    | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                        log::info!("Detected sound file removal: {}", file.display());
                        let event = SoundEvent::Remove(file);
                        out_tx.send(event).unwrap();
                    }
                    EventKind::Modify(ModifyKind::Name(_)) => {
                        // On macOS, file deletions can show up as Modify(Name(Any))
                        // Check if the file still exists to determine if it was deleted
                        if file_exists {
                            log::info!("Detected sound file creation (via name modify): {}", file.display());
                            let event = SoundEvent::Add(file);
                            out_tx.send(event).unwrap();
                        } else {
                            log::info!("Detected sound file removal (via name modify): {}", file.display());
                            let event = SoundEvent::Remove(file);
                            out_tx.send(event).unwrap();
                        }
                    }
                    _ => {
                        log::debug!("Ignoring event kind: {:?}", event.kind);
                    }
                }
            },
            notify::Config::default(),
        )
        .unwrap();

        tokio::spawn(Self::fan_out(out_rx, to_fan));

        tokio::spawn(Self::fan_in(in_rx, handler));

        Self { in_tx, events }
    }

    pub fn push_files(
        &mut self,
        backend_tx: mpsc::Sender<crate::ui::BackendToFrontendMessage>,
    ) -> Result<(), std::io::Error> {
        let mut rx = self.events.clone();
        tokio::spawn(async move {
            log::info!("Sound file watcher task started");
            loop {
                if let Err(e) = rx.changed().await {
                    log::error!("Watcher channel error: {}", e);
                    break;
                };

                // Process events in a scoped block to ensure locks are dropped
                let has_changes = {
                    let mut lock = FILES.lock().unwrap();
                    let events = rx.borrow();
                    let mut changed = false;

                    log::debug!("Processing {} sound file events", events.len());

                    for event in events.iter() {
                        let get_filename = || -> Option<&str> {
                            match event {
                                SoundEvent::Add(file) | SoundEvent::Remove(file)
                                    if Soundlist::is_soundfile(&file).is_some() =>
                                {
                                    file.file_stem()?.to_str()
                                }
                                _ => None,
                            }
                        };

                        if let Some(filename) = get_filename() {
                            changed = true;
                            match event {
                                SoundEvent::Add(_) => {
                                    log::info!("Added sound file: {}", filename);
                                    lock.insert(String::from(filename));
                                }
                                SoundEvent::Remove(_) => {
                                    log::info!("Removed sound file: {}", filename);
                                    lock.remove(filename);
                                }
                            }
                        }
                    }

                    changed
                }; // lock and events are dropped here

                // Save the updated soundlist to file and notify UI if there were changes
                if has_changes {
                    if let Err(e) = Soundlist::save_from_files().await {
                        log::error!("Failed to save soundlist: {}", e);
                    }
                    // Notify the UI that the sound list has been updated
                    if let Err(e) = backend_tx
                        .send(crate::ui::BackendToFrontendMessage::SFXListUpdated)
                        .await
                    {
                        log::error!("Failed to send SFXListUpdated message: {}", e);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.in_tx.send(path.to_path_buf())?;
        Ok(())
    }

    async fn fan_in(mut rx: watch::Receiver<PathBuf>, mut handler: impl notify::Watcher) {
        loop {
            handler
                .watch(&rx.borrow_and_update(), notify::RecursiveMode::NonRecursive)
                .unwrap();

            // for unwinding when panicking
            assert!(rx.changed().await.is_err());
        }
    }

    async fn fan_out(
        rx: mpsc::UnboundedReceiver<SoundEvent>,
        tx: watch::Sender<HashSet<SoundEvent>>,
    ) {
        let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(100));
        pin!(rx);

        while let Some(chunk) = rx.next().await {
            let events: HashSet<_> = chunk.into_iter().collect();
            let _ = tx.send(events);
        }
    }
}
