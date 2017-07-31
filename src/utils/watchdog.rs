#[macro_use]
use std::thread;
use std::path::{PathBuf, Path};
use std::time::{Duration, Instant};
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use notify;
use notify::{Watcher, RecursiveMode, DebouncedEvent, RecommendedWatcher};
use super::ssh;


pub struct WatchDog <'a, P>
where P: AsRef<Path>{
    pub src_path: P,
    pub dest_root: P,
    pub ignore_paths: Option<Vec<P>>,
    pub tx: Sender<DebouncedEvent>,
    pub rx: Receiver<DebouncedEvent>,
    pub timeout: u64,
    pub sftp: &'a ssh::SftpClient<'a>,
}

pub trait watch{
    fn handle_events(&mut self, event: &DebouncedEvent);
}

impl <'a, T> WatchDog<'a, T>
where T: AsRef<Path>{

    fn watch(& mut self) {
        // block to wait file change
        match self.rx.recv() {
            Ok(event) => { self.handle_events(&event);},
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    pub fn start(&mut self) -> notify::Result<()>{
        let mut watcher: RecommendedWatcher = Watcher::new(self.tx.clone(), Duration::from_secs(2))?;
        watcher.watch(self.src_path.as_ref(), RecursiveMode::Recursive)?;
        match watcher.unwatch("/Users/yuanlinfeng/Desktop/cloud/.git") {
            Err(e) => println!("error to unwatch: {:?}", e),
            Ok(_) => (),
        }
        loop {
            self.watch();
        }
    }
}
