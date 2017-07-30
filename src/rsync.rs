extern crate fsevent;

use std::thread;
use std::sync::mpsc::channel;
use std::process::Command;
use std::path::Path;
use std::env;

fn main() {

    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("rs-auto-sync local_path remote_path");
        return;
    }

    let local_path_string = args[1].clone();
    let remote_path_string = args[2].clone();
    let local_path = Path::new(&local_path_string);

    if !local_path.starts_with("/") {
        println!("plz rewrite local_path as absolute path");
        return;
    }

    rsync(&local_path_string, &remote_path_string);

    //watch fs event
    let (event_tx, event_rx) = channel();
    thread::spawn(move || {
        let fsevent = fsevent::FsEvent::new(event_tx);
        fsevent.append_path(&args[1]);
        fsevent.observe();
    });

    // fs event handled here
    loop {
        let result = event_rx.recv();
        if !result.is_ok(){
            continue;
        }
        let event = result.unwrap();
        // ignore hiden files
        if !event.path.find("/.").is_none() {
            continue;
        }

        println!("{:?}", event);

        if event.flag.contains(fsevent::ITEM_REMOVED) ||
            event.flag.contains(fsevent::ITEM_RENAMED) {
            rsync(&local_path_string, &remote_path_string);
            continue;
        }

        let event_path = Path::new(&event.path);
        let parent_event_path = event_path.parent().unwrap();
        let parent_local_path = local_path.parent().unwrap();
        let target_remote_path = parent_event_path
            .to_str().unwrap()
            .replace(parent_local_path.to_str().unwrap(), &remote_path_string);
        rsync(&event.path, &target_remote_path);
    }
}


fn rsync (source :&str, target :&str) {
    println!(">> rsync {} {}", source, target);
    let options = vec![
        "-r",
        "-v",
        "--exclude=.[a-zA-Z0-9]*",
        "--filter=:- .gitignore",
        "--delete"];
    let output = Command::new("rsync")
        .args(&options)
        .arg(source)
        .arg(target)
        .output()
        .unwrap_or_else(|e| {
            panic!("failed to execute process: {}", e)
        });
    if output.stdout.len() > 0 {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
    if output.stderr.len() > 0 {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
