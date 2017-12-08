use errors::*;
use std::path::{PathBuf, Path};
use std::time::{Duration};
use std::sync::mpsc::{Sender, Receiver};
use notify::{self, Watcher, RecursiveMode, DebouncedEvent, RecommendedWatcher};
use super::ssh;
use regex::Regex;
use std::fs;
use utils::util::is_exclude;
use std::os::unix::fs::PermissionsExt;

pub struct WatchDog <'a,'b> {
    pub src_path: &'b Path,
    pub dest_root: &'b Path,
    pub tx: Sender<DebouncedEvent>,
    pub rx: Receiver<DebouncedEvent>,
    pub sftp: &'a ssh::SftpClient<'a>,
    pub exclude_files: &'b mut Vec<PathBuf>,
    pub include_files: &'b mut Vec<PathBuf>,
    pub re_vec: &'b Vec<Regex>,
}


impl <'a, 'b> WatchDog<'a, 'b> {
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

    // 得到目标文件
    pub fn get_dest_path_buf(&self, path: &Path) ->  Result<PathBuf>{
        let p = path.strip_prefix(self.src_path)?;
        let dest_path = self.dest_root.join(p);
        Ok(dest_path.to_path_buf())
    }

    fn watch(& mut self) {
        // block to wait file change
        match self.rx.recv() {
            Ok(event) => { self.handle_events(&event); },
            Err(e) => error!("watch error: {:?}", e),
        }
    }

    pub fn start(&mut self) -> notify::Result<()>{
        let mut watcher: RecommendedWatcher = Watcher::new(self.tx.clone(), Duration::from_secs(2))?;
        watcher.watch(self.src_path, RecursiveMode::Recursive)?;
//        match watcher.unwatch("/Users/yuanlinfeng/Desktop/cloud/.git") {
//            Err(e) => println!("error to unwatch: {:?}", e),
//            Ok(_) => (),
//        }
        loop {
            self.watch();
        }
    }

    fn do_handle_events(&mut self, event: &DebouncedEvent) -> Result<()>{
        match event {
            &DebouncedEvent::NoticeWrite(ref path) => {info!("notice write: {:?}", path);},
            &DebouncedEvent::NoticeRemove(ref path) => {info!("notice remove: {:?}", path);},
            &DebouncedEvent::Create(ref path) => {
                if self.exclude_files.iter().any(|r| *r == path.to_path_buf()) {
                    info!(" ' {:?} ' created, but ignored!", path);
                    return Ok(())
                } else if !self.include_files.iter().any(|r| *r == path.to_path_buf()) {
                    if is_exclude(path, self.re_vec) {
                        self.exclude_files.push(path.to_path_buf())
                    } else {
                        self.exclude_files.push(path.to_path_buf())
                    }
                }
                if self.include_files.iter().any(|r| *r == path.to_path_buf()) {
                    let dest_path_buf = self.get_dest_path_buf(path)?;
                    let dest_path = dest_path_buf.as_path();
                    info!("notice create: {:?}, get dest path:{:?}", path, dest_path);
                    let file_type = fs::metadata(path)?.file_type();
                    if file_type.is_dir() {
                        // get mode from src path
                        let permissions = fs::metadata(path)?.permissions();
                        let mode = permissions.mode() as i32; // return u32
                        self.sftp.mkdir(dest_path, mode)?;
                    } else if file_type.is_file() {
                        self.sftp.upload_file(path, dest_path)?;
                    } else if file_type.is_symlink() {
                        // TODO: get the real path of the link
                        //let dest_src = "";
                        //self.sftp.symlink(dest_src, dest_path)?;
                    }
                }
            },
            &DebouncedEvent::Write(ref path) => {info!("notice write: {:?}", path);},
            &DebouncedEvent::Chmod(ref path) => {info!("notice chmod: {:?}", path);},
            &DebouncedEvent::Remove(ref path) => {info!("notice remove: {:?}", path);},
            &DebouncedEvent::Rename(ref path_src, ref path_dest) => {info!("notice rename : {:?} -> {:?}", path_src, path_dest);},
            &DebouncedEvent::Rescan => {},
            &DebouncedEvent::Error(ref e, ref path) => {info!("error {:?}: {:?}", &path, e)},
        }
        Ok(())
    }
}
