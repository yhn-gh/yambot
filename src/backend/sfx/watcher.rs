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
                    return log::error!("Watcher Error: {:?}", res);
                };

                // pop removes first parent
                let file = event.paths.pop().expect("No Such file exists");

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                        let event = SoundEvent::Add(file);
                        out_tx.send(event).unwrap();
                    }
                    EventKind::Remove(_)
                    | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                        let event = SoundEvent::Remove(file);
                        out_tx.send(event).unwrap();
                    }
                    _ => (),
                }
            },
            notify::Config::default(),
        )
        .unwrap();

        tokio::spawn(Self::fan_out(out_rx, to_fan));

        tokio::spawn(Self::fan_in(in_rx, handler));

        Self { in_tx, events }
    }

    pub fn push_files(&mut self) -> Result<(), std::io::Error> {
        let mut rx = self.events.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = rx.changed().await {
                    log::error!("Watcher channel error: {}", e);
                    break;
                };
                let mut lock = FILES.lock().unwrap();
                let events = rx.borrow();
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
                        match event {
                            SoundEvent::Add(file) => {
                                log::info!("Added sound file: {}", file.display());
                                lock.insert(filename.to_owned());
                            }
                            SoundEvent::Remove(file) => {
                                log::info!("Removing sound file: {}", file.display());
                                lock.remove(filename);
                            }
                        }
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
