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

use super::sfx::{Sounds, FILES};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SoundEvent {
    Add(String),
    Remove(String),
}

impl Watcher {
    pub fn serve() -> Self {
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        let (in_tx, in_rx) = watch::channel(PathBuf::new());
        let (to_fan, events) = watch::channel(HashSet::new());

        let handler = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                let Ok(mut event) = res else {
                    return log::error!("Watcher Error: {:?}", res);
                };

                let file = event.paths.pop().expect("No Such file exists");
                let file = file.file_name().unwrap().to_string_lossy();

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                        let event = SoundEvent::Add(file.to_string());
                        out_tx.send(event).unwrap();
                    }
                    EventKind::Remove(_)
                    | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                        let event = SoundEvent::Remove(file.to_string());
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
                if rx.changed().await.is_err() {
                    log::info!("Watcher channel closed");
                    break;
                }

                let events = rx.borrow();
                for event in events.iter() {
                    Self::handle_event(event)
                }
            }
        });
        Ok(())
    }

    fn handle_event(event: &SoundEvent) {
        match event {
            SoundEvent::Add(path) => {
                if let Some(filename) = Sounds::get_filename(path.to_string()) {
                    log::info!("Added sound file: {}", filename);
                    FILES.lock().unwrap().insert(filename);
                }
            }
            SoundEvent::Remove(path) => {
                if let Some(filename) = Sounds::get_filename(path.to_string()) {
                    log::info!("Remove sound file: {}", filename);
                    FILES.lock().unwrap().remove(&filename);
                }
            }
        }
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

            if rx.changed().await.is_err() {
                break;
            }
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
