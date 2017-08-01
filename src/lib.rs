pub mod utils;
pub mod errors;

extern crate glob;
extern crate ssh2;
extern crate serde;
extern crate notify;
extern crate regex;
extern crate toml;
extern crate shellexpand;
#[macro_use]extern crate error_chain;
#[macro_use] extern crate serde_derive;

use utils::*;
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::sync::mpsc:: channel;
use std::fs;
use std::io;
use glob::glob;
use shellexpand::{tilde, tilde_with_context};
use notify::DebouncedEvent;


#[cfg(not(target_os="windows"))] const TIMEOUT_MS: u64 = 100;
#[cfg(target_os="windows")] const TIMEOUT_MS: u64 = 2000; // windows can take a while

fn get_dest_path<'a, P>(path: &'a P, src_path: &'a P, dest_root: &'a P) ->  Option<String>
where P: AsRef<Path> {
    match path.as_ref().strip_prefix(src_path) {
        Ok(ref p) => {
            let dest_path = dest_root.as_ref().join(p);
            Some(dest_path.to_str().unwrap().to_string())
        },
        Err(e) => {
            println!("can not get post_fix for {:?}", src_path.as_ref());
            None
        },
    }
}

// 处理文件更改事件
impl <'a, P> watchdog::watch for watchdog::WatchDog<'a, P>
where P: AsRef<Path>{
    fn handle_events(&mut self, event: &DebouncedEvent){
        match event {
            &DebouncedEvent::NoticeWrite(ref path) => {println!("notice write: {:?}", path);},
            &DebouncedEvent::NoticeRemove(ref path) => {println!("notice remove: {:?}", path);},
            &DebouncedEvent::Create(ref path) => {println!("notice create: {:?}", path);},
            &DebouncedEvent::Write(ref path) => {println!("notice write: {:?}", path);},
            &DebouncedEvent::Chmod(ref path) => {println!("notice chmod: {:?}", path);},
            &DebouncedEvent::Remove(ref path) => {println!("notice rename: {:?}", path);},
            &DebouncedEvent::Rename(ref path_src, ref path_dest) => {println!("notice : {:?} -> {:?}", path_src, path_dest);},
            &DebouncedEvent::Rescan => {},
            &DebouncedEvent::Error(ref e, ref path) => {println!("error {:?}: {:?}", &path, e)},
        }
    }
}

// 获取忽略文件
fn get_dir_ignored<P, S>(root: P, exclude: Option<&Vec<S>>, ignore_path: &mut Vec<String>) -> io::Result<()>
where P: AsRef<Path> + PartialEq, S: AsRef<str>{
    if !root.as_ref().metadata()?.file_type().is_dir() {
        println!("the root path is not directory!");
    } else {
        for ipath in exclude.unwrap() {
            let temp_path = root.as_ref().join(Path::new(ipath.as_ref()));
            let temp_path_str = temp_path.to_str().unwrap();
            for entry in glob(temp_path_str).unwrap() {
                match entry {
                    Ok(path) => ignore_path.push(path.to_str().unwrap().into()),
                    Err(e) => println!("error when get glob path: {:?}", e),
                }
            }

        }
        for entry in fs::read_dir(root.as_ref()).unwrap() {
            let entry = entry?;
            let path = entry.path();
            if ignore_path.iter().any(|r| r.as_str()==path.to_str().unwrap_or("")){
                continue;
            } else {
                println!("find unignored path: {:?}", path);
            }
            if entry.file_type()?.is_dir() {
                println!("{} is directory", path.display());
                get_dir_ignored(&path, exclude, ignore_path);
            } else if entry.file_type()?.is_file(){
                println!("{} is file", path.display());
            } else if entry.file_type()?.is_symlink(){
                println!("{} is symlink", path.display());
            } else {
                println!("{} is unknown type", path.display())
            }
        }
    }
    Ok(())
}


fn start_watch<P: AsRef<Path>>(src_path: P, dest_root: P, sftp: &ssh::SftpClient, ignore_paths: Option<Vec<P>>) {
    println!("watching path: {:?}", src_path.as_ref());
    let (tx, rx) = channel();
    let mut watchdog = watchdog::WatchDog {
        src_path: src_path,
        dest_root: dest_root,
        tx: tx,
        rx:rx,
        timeout: TIMEOUT_MS,
        sftp: sftp,
        ignore_paths: ignore_paths,
    };
    watchdog.start();
}


pub fn run<S, P>(config_path: P, project_name: S, server: S, watch: bool)
    where S: AsRef<str> + Debug + PartialEq,
          P: AsRef<Path> + Debug
{
    // get the global config
    let global_config = toml_parser::get_config(config_path).unwrap();
    println!("1. =================================\n\n");
    println!("{:?}", global_config);
    let project = toml_parser::get_project_info(project_name, &global_config).unwrap();
    println!("2. =================================\n\n");
    println!("{:?}", project);

    // get host config
    let ssh_conf_path = tilde("~/.ssh/config").into_owned();

    let server_host = sshconfig::parse_ssh_config(ssh_conf_path).unwrap();
    let host:sshconfig::Host = match server_host.get(server.as_ref()) {
        Some(host) => {
            let mut host = host.clone();
            if host.IdentityFile.is_none() {
                host.Password = global_config.global_password;
                if global_config.global_key.is_some() {
                    host.IdentityFile = Some(Path::new(global_config.global_key.unwrap().as_str()).into());
                }
            }
            host
        },
        None => {
            //let HostName = sshconfig::get_ip();
            let HostName = sshconfig::get_ip(server.as_ref());
            let User = global_config.global_user;
            let IdentityFile = match global_config.global_key{
                None => None,
                Some(ref file) => Some(tilde(file).into_owned())
            };
            let Password = global_config.global_password;
            let Port = global_config.global_port;
            sshconfig::Host::new(HostName, User, IdentityFile, Password, Port)
        }
    };
    println!("3. =================================\n\n");
    println!("{:?}", host);

    // connect
    let user = host.User.clone();
    let sshclient = ssh::SSHClient::new(host.HostName, host.Port, host.User, host.Password, host.IdentityFile);
    println!("{:?}",sshclient.run_cmd("ls /tmp"));
    let sftpclient = ssh::SftpClient::new(&sshclient);

    // change ~ to /home/user or /root in dest path
    let common_home = format!("/home/{}", user);
    let dest_root = tilde_with_context(project.dest.as_str(), ||{
        if user == "root".to_string() {
            Some(Path::new("/root").into())
        } else {
            Some(Path::new(&common_home))
        }
    }).into_owned();
    println!("dest path: {}", dest_root);

    // get ignore dir
    let mut V = Vec::new();
    get_dir_ignored(&project.src, project.exclude.as_ref(), &mut V);

    //start watch
    let ignore_paths = if V.len() > 0 {Some(V)} else {None};
    start_watch(project.src, dest_root, &sftpclient, ignore_paths);
}
