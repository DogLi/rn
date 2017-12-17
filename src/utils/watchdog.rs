use errors::*;
use std::path::{PathBuf, Path};
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use notify::{self, Watcher, RecursiveMode, DebouncedEvent, RecommendedWatcher};
use super::ssh;
use regex::Regex;
use std::fs;
use utils::util;
use std::os::unix::fs::PermissionsExt;

pub struct WatchDog<'a, 'b> {
    pub src_path: &'b Path,
    pub dest_root: &'b Path,
    pub tx: Sender<DebouncedEvent>,
    pub rx: Receiver<DebouncedEvent>,
    pub sftp: &'a ssh::SftpClient<'a>,
    pub exclude_files: &'b mut Vec<PathBuf>,
    pub include_files: &'b mut Vec<PathBuf>,
    pub re_vec: &'b Vec<Regex>,
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

    // 得到目标文件
    pub fn get_dest_path_buf(&self, path: &Path) -> Result<PathBuf> {
        debug!("path: {:?}", path);
        let p = path.strip_prefix(self.src_path)?;
        debug!("strip_prefix path: {:?}", p);
        let dest_path = self.dest_root.join(p);
        debug!("dest path: {:?}", dest_path);
        Ok(dest_path.to_path_buf())
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
        watcher.watch(self.src_path, RecursiveMode::Recursive)?;
        loop {
            self.watch();
        }
    }

    fn do_handle_events(&mut self, event: &DebouncedEvent) -> Result<()> {
        match event {
//            &DebouncedEvent::NoticeWrite(ref path) => {
//                info!("notice write: {:?}", path);
//            }
//            &DebouncedEvent::NoticeRemove(ref path) => {
//                info!("notice remove: {:?}", path);
//            }
            &DebouncedEvent::Create(ref path) => {
               /* if self.exclude_files.iter().any(|r| *r == path.to_path_buf()) {
                    info!(" ' {:?} ' created, but ignored!", path);
                    return Ok(());
                } else if !self.include_files.iter().any(|r| *r == path.to_path_buf()) {
                    if util::is_exclude(path, self.re_vec) {
                        self.exclude_files.push(path.to_path_buf())
                    } else {
                        self.exclude_files.push(path.to_path_buf())
                    }
                }*/

                if util::is_exclude(path.as_path(), self.re_vec) {
                    self.exclude_files.push(path.to_path_buf());
                    return Ok(())
                } else {
                    self.include_files.push(path.to_path_buf());
                }
                debug!("path: {:?}", path);
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
                    let src_path = fs::read_link(path)?;
                    let dest_src = self.get_dest_path_buf(&src_path)?;
                    self.sftp.symlink(&dest_src.as_path(), &dest_path)?;
                }
            }
            &DebouncedEvent::Write(ref path) => {
                if self.exclude_files.iter().any(|r| *r == path.to_path_buf()) {
                    info!(" ' {:?} ' changed, upload it", path);
                    return Ok(());
                } else {
                    let dest_path_buf = self.get_dest_path_buf(path)?;
                    let dest_path = dest_path_buf.as_path();
                    self.sftp.upload_file(path, dest_path)?;
                    info!("notice write: {:?}", path);
                }
            }
            &DebouncedEvent::Chmod(ref path) => {
                if self.exclude_files.iter().any(|r| *r == path.to_path_buf()) {
                    info!(" ' {:?} ' chown, reset it", path);
                    return Ok(());
                } else {
                    info!("' {:?} ' chown, but ignored!", path);
                }
            }
            &DebouncedEvent::Remove(ref path) => {
                if self.exclude_files.iter().any(|r| *r == path.to_path_buf()) {
                    info!(" ' {:?} ' removed, remove it", path);
                    return Ok(());
                } else {
                    info!("' {:?} ' removed, but ignored!", path);
                }
            }
            &DebouncedEvent::Rename(ref path_src, ref path_dest) => {
                info!("rename : {:?} -> {:?}", path_src, path_dest);
                // remove the source path
                if self.exclude_files.iter().any(|r| *r == path_src.to_path_buf()) {
                    self.exclude_files.retain(|ref x| **x == path_src.to_path_buf());
                } else if self.include_files.iter().any(|r| *r == path_src.to_path_buf())  {
                    self.include_files.retain(|ref x| **x == path_src.to_path_buf());
                }
                // check the path_dest is in include_files or exclude_files

            }
            &DebouncedEvent::Rescan => {}
            &DebouncedEvent::Error(ref e, ref path) => info!("error {:?}: {:?}", &path, e),
            _ => {} // &NoticeWrite and &NoticeRemove
        }
        Ok(())
    }

    fn do_create(&mut self) -> Result<()>{
        // pass
        Ok(())
    }

    fn do_remove(&mut self) -> Result<()> {
        Ok(())
    }

    fn do_link(&mut self) -> Result<()> {
        Ok(())
    }

    fn do_chmod(&mut self) -> Result<()> {
        Ok(())
    }

    fn do_write(&mut self) -> Result<()> {
        Ok(())
    }

    fn do_rename(&mut self) -> Result<()> {
        Ok(())
    }
}
