#[macro_use]
use notify::*;
use std::thread;
use std::path::{PathBuf, Path};
use std::time::{Duration, Instant};
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher, op};
use super::ssh;


pub struct WatchDog <'a, P>
where P: AsRef<Path>{
    pub src_path: P,
    pub dest_root: P,
    pub ignore_paths: Option<Vec<P>>,
    pub tx: Sender<RawEvent>,
    pub rx: Receiver<RawEvent>,
    pub events: Vec<(PathBuf, Op, Option<u32>)>,
    pub timeout: u64,
    pub sftp: &'a ssh::SftpClient<'a>,
}


// FSEvents tends to emit events multiple times and aggregate events,
// so just check that all expected events arrive for each path,
// and make sure the paths are in the correct order
pub fn inflate_events(input: &Vec<(PathBuf, Op, Option<u32>)>) -> Vec<(PathBuf, Op, Option<u32>)> {
    let mut output = Vec::new();
    let mut path = None;
    let mut ops = Op::empty();
    let mut cookie = None;

    for &(ref e_p, e_o, e_c) in input {
        let p = match path {
            Some(p) => p,
            None => e_p.clone()
        };
        let c = match cookie {
            Some(c) => Some(c),
            None => e_c
        };
        if p == *e_p && c == e_c {
            ops |= e_o;
        } else {
            output.push((p, ops, cookie));
            ops = e_o;
        }
        path = Some(e_p.clone());
        cookie = e_c;
    }
    if let Some(p) = path {
        output.push((p, ops, cookie));
    }
    output
}


pub trait watch{
    fn handle_events(&mut self);
}

impl <'a, T> WatchDog<'a, T>
where T: AsRef<Path>{
    fn recv_events_with_timeout(&mut self) {
        let start = Instant::now();
        let timeout = Duration::from_millis(self.timeout);

        while start.elapsed() < timeout {
            match self.rx.try_recv() {
                Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                    self.events.push((path, op, cookie));
                },
                Ok(RawEvent{path: None, ..})  => (),
                Ok(RawEvent{op: Err(e), ..}) => panic!("unexpected event err: {:?}", e),
                Err(TryRecvError::Empty) => (),
                Err(e) => panic!("unexpected channel err: {:?}", e)
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn watch(& mut self) {
        // block to wait file change
        match self.rx.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                self.events.push((path, op, cookie));
                self.recv_events_with_timeout();
                if cfg!(target_os="windows") {
                    // Windows may sneak a write event in there
                    self.events.retain(|&(_, op, _)| op != op::WRITE);
                } else if cfg!(target_os="macos") {
                    let events = inflate_events(&self.events);
                    self.events = events;
                };
                self.handle_events();
            },
            Ok(event) => println!("broken event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    pub fn start(&mut self) {
        let mut watcher = raw_watcher(self.tx.clone()).unwrap();
        watcher.watch(self.src_path.as_ref(), RecursiveMode::Recursive).unwrap();

        // set unwatch files
        for i_path in self.ignore_paths.as_ref().unwrap() {
            println!("uuuuuuuuuuuuuuuuuuuuuu: {:?}", i_path.as_ref());
            watcher.unwatch(i_path.as_ref());
        }
        let path = Path::new("/Users/yuanlinfeng/Desktop/cloud/.git/index.lock");
        watcher.unwatch(path);

        loop {
            self.watch();
        }
    }
}
