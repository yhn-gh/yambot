use tokio::sync::{mpsc, watch};
use notify::{RecommendedWatcher, Watcher as _Watcher};
use std::path::{Path, PathBuf};
use std::error::Error;
use super::sfx::SoundList;

pub struct Watcher {
    in_tx: watch::Sender<PathBuf>,
    out_tx: mpsc::UnboundedSender<notify::Event>,
}

impl Watcher {
    pub fn serve() -> Self {
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        let (in_tx, in_rx) = watch::channel(PathBuf::new());
        let out_tx_ = out_tx.clone();

        let handler = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                let Ok(event) = res else {
                    return log::error!("Watcher Error: {:?}", res);
                };
                if let notify::EventKind::Create(_) = event.kind {
                    out_tx_.send(event).unwrap();
                }
            },
            notify::Config::default(),
        )
        .unwrap();

        tokio::spawn(Self::fan_out(out_rx));

        tokio::spawn(Self::fan_in(in_rx, handler));

        Self { in_tx, out_tx }
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), Box<dyn Error>>{
        self.in_tx.send(path.to_path_buf())?;
        Ok(())
    }

    async fn fan_in(mut rx: watch::Receiver<PathBuf>, mut handler: impl notify::Watcher) {
        loop {
            let path = rx.borrow_and_update();
            handler
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .unwrap();
            }
    }

    async fn fan_out(mut rx: mpsc::UnboundedReceiver<notify::Event>) {
        while let Some(mut rx) = rx.recv().await {
            let mut stuff = SoundList::new(Path::new("/home/yhn/test/sfx/soundlist.json")).unwrap();
            let path = rx.paths.pop().unwrap();
            stuff.add(Path::new(&path)).unwrap();
        }
    }
}
