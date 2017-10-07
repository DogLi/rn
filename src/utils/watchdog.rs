use errors::*;
use std::path::{PathBuf, Path};
use std::time::{Duration};
use std::sync::mpsc::{Sender, Receiver};
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
    pub sftp: &'a ssh::SftpClient<'a>,
}

pub trait Watch{
    fn do_handle_events(&mut self, event: &DebouncedEvent) -> Result<()>;
}


impl <'a, T> WatchDog<'a, T>
where T: AsRef<Path>{
    fn handle_events(&mut self, event: &DebouncedEvent) {
        if let Err(ref e) = self.do_handle_events(event) {
            println!("error: {}", e);

            for e in e.iter().skip(1) {
                println!("caused by: {}", e);
            }

            if let Some(backtrace) = e.backtrace() {
                println!("backtrace: {:?}", backtrace);
            }
        }
    }

    // 得到目标文件
    pub fn get_dest_path_buf<P: AsRef<Path>>(&self, path: &P) ->  Result<PathBuf>{
        let p = path.as_ref().strip_prefix(self.src_path.as_ref())?;
        let dest_path = self.dest_root.as_ref().join(p);
        Ok(dest_path.to_path_buf())
    }

    fn watch(& mut self) {
        // block to wait file change
        match self.rx.recv() {
            Ok(event) => { self.handle_events(&event); },
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
