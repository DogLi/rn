use errors::*;
use super::{sshconfig, rsync, toml_parser};
use std::path::Path;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use notify::{self, Watcher, RecursiveMode, DebouncedEvent, RecommendedWatcher};

pub struct WatchDog<'a, 'b> {
    pub project: &'a toml_parser::Project,
    pub host: &'b sshconfig::Host,
    pub tx: Sender<DebouncedEvent>,
    pub rx: Receiver<DebouncedEvent>,
}


impl<'a, 'b> WatchDog<'a, 'b> {
    fn handle_events(&mut self, event: &DebouncedEvent) {
        if let Err(ref e) = self.do_handle_events(event) {
            error!("error: {}", e);
            for e in e.iter().skip(1) {
                error!("caused by: {}", e);
            }
            if let Some(backtrace) = e.backtrace() {
                error!("backtrace: {:?}", backtrace);
            }
        }
    }

    fn watch(&mut self) {
        // block to wait file change
        match self.rx.recv() {
            Ok(event) => {
                self.handle_events(&event);
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }

    pub fn start(&mut self) -> notify::Result<()> {
        let mut watcher: RecommendedWatcher =
            Watcher::new(self.tx.clone(), Duration::from_secs(2))?;
        watcher.watch(self.project.src.as_str(), RecursiveMode::Recursive)?;
        loop {
            self.watch();
        }
    }

    fn do_handle_events(&mut self, event: &DebouncedEvent) -> Result<()> {
        match event {
            &DebouncedEvent::NoticeWrite(ref path) | &DebouncedEvent::NoticeRemove(ref path)=> {
                info!("do nothing");
            }
            _ => {
                rsync::sync(
                    self.host,
                    self.project,
                    true,
                )?;
                info!("do rsync!");
            }
        }
        Ok(())
    }
}
