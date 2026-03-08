use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<()>,
}

impl FileWatcher {
    pub fn new(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();

        let watched_path = std::fs::canonicalize(path)?;
        let event_tx = tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            let _ = event_tx.send(());
                        }
                        _ => {}
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        // Watch the parent directory (more reliable for file replacements)
        let parent = watched_path.parent()
            .ok_or("Cannot watch file: no parent directory")?;
        watcher.watch(parent.as_ref(), RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Check if a file change event has been received (non-blocking).
    pub fn poll_change(&self) -> bool {
        // Drain all pending events, return true if any existed
        let mut changed = false;
        while self.rx.try_recv().is_ok() {
            changed = true;
        }
        changed
    }
}
