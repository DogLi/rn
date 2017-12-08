pub mod utils;
pub mod errors;
pub mod my_logger;

extern crate glob;
extern crate ssh2;
extern crate serde;
extern crate notify;
extern crate regex;
extern crate toml;
extern crate shellexpand;


#[macro_use(slog_o, slog_debug, slog_info, slog_warn, slog_error, slog_crit, slog_log, slog_record, slog_record_static, slog_b, slog_kv)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
extern crate slog_json;
extern crate slog_async;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;

use errors::*;
use utils::*;
use std::path::Path;
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::sync::mpsc:: channel;
use std::fs;
use glob::glob;
use shellexpand::{tilde, tilde_with_context};
use notify::DebouncedEvent;
use std::os::unix::fs::PermissionsExt;


impl <'a, P> watchdog::Watch for watchdog::WatchDog<'a, P>
where P: AsRef<Path>{
    // 处理文件更改事件

    fn do_handle_events(&mut self, event: &DebouncedEvent) -> Result<()>{
        match event {
            &DebouncedEvent::NoticeWrite(ref path) => {println!("notice write: {:?}", path);},
            &DebouncedEvent::NoticeRemove(ref path) => {println!("notice remove: {:?}", path);},
            &DebouncedEvent::Create(ref path) => {
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

// 获取忽略文件
fn get_dir_ignored<P, S>(root: P, exclude: Option<&Vec<S>>, ignore_path: &mut Vec<String>) -> Result<()>
where P: AsRef<Path> + PartialEq, S: AsRef<str>{
    if !root.as_ref().metadata()?.file_type().is_dir() {
        info!("the root path is not directory!");
    } else {
        for ipath in exclude.unwrap() {
            let temp_path_str = root.as_ref().join(Path::new(ipath.as_ref())).to_str().unwrap();
//          let temp_path_str = temp_path.to_str().unwrap();
            for entry in glob(temp_path_str).unwrap() {
                match entry {
                    Ok(path) => ignore_path.push(path.to_str().unwrap().into()),
                    Err(e) => error!("error when get glob path: {:?}", e),
                }
            }

        }
        for entry in fs::read_dir(root.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if ignore_path.iter().any(|r| r.as_str()==path.to_str().unwrap_or("")){
                continue;
            } else {
                info!("find unignored path: {:?}", path);
            }
            if entry.file_type()?.is_dir() {
                info!("{} is directory", path.display());
                get_dir_ignored(&path, exclude, ignore_path)?;
            } else if entry.file_type()?.is_file(){
                info!("{} is file", path.display());
            } else if entry.file_type()?.is_symlink(){
                info!("{} is symlink", path.display());
            } else {
                warn!("{} is unknown type", path.display())
            }
        }
    }
    Ok(())
}


fn start_watch<P: AsRef<Path>>(src_path: P, dest_root: P, sftp: &ssh::SftpClient, ignore_paths: Option<Vec<P>>) -> Result<()>{
    info!("watching path: {:?}", src_path.as_ref());
    let (tx, rx) = channel();
    let mut watchdog = watchdog::WatchDog {
        src_path: src_path,
        dest_root: dest_root,
        tx: tx,
        rx:rx,
        sftp: sftp,
        ignore_paths: ignore_paths,
    };
    watchdog.start()?;
    Ok(())
}


pub fn run<S, P>(config_path: P, project_name: S, server: S, watch: bool, user: Option<S>, password: Option<S>, identity: Option<S>) -> Result<()>
    where S: AsRef<str> + Debug + PartialEq,
          P: AsRef<Path> + Debug
{
    let log = slog_scope::logger();
    // get the global config
    let global_config = toml_parser::get_config(config_path)?;
    info!("{:?}", global_config);
    let project = toml_parser::get_project_info(&project_name, &global_config)?;
    info!("{:?}", project);

    // get host config
    let ssh_conf_path = tilde("~/.ssh/config").into_owned();

    let server_host = sshconfig::parse_ssh_config(ssh_conf_path)?;
    let mut host: sshconfig::Host = match server_host.get(server.as_ref()) {
        Some(host) => {
            let mut host = host.clone();
            if host.identityfile.is_none() {
                // TODO: get password from input or get key file from input
                host.password = global_config.global_password;
                if global_config.global_key.is_some() {
                    host.identityfile = Some(Path::new(global_config.global_key.unwrap().as_str()).into());
                }
            }
            host
        },
        None => {
            //let hostname = sshconfig::get_ip();
            let hostname = sshconfig::get_ip(server.as_ref())?;
            let g_user = global_config.global_user;
            // TODO: get password or key file from input
            let identityfile = match global_config.global_key{
                None => None,
                Some(ref file) => Some(tilde(file).into_owned())
            };
            let g_password = global_config.global_password;
            let port = global_config.global_port;
            sshconfig::Host::new(hostname, g_user, identityfile, g_password, port)
        }
    };

    // update user, password, identity file
    if user.is_some() {
        host.user = user.unwrap().as_ref().to_string();
    }
    if password.is_some() {
        host.password = Some(password.unwrap().as_ref().to_string());
        host.identityfile = None;
    }
    if identity.is_some() {
        host.identityfile = Some(Path::new(identity.unwrap().as_ref()).to_path_buf());
        host.password = None;
    }

    info!("{:?}", host);

    // connect
    let user = host.user.clone();
    let sshclient = ssh::SSHClient::new(host.hostname, host.port, host.user, host.password, host.identityfile)?;
    let cmd_output = sshclient.run_cmd("ls /tmp")?;
    info!("{:?}", cmd_output);
    let sftpclient = ssh::SftpClient::new(&sshclient);

    // change ~ to /home/user or /root in dest path
    let common_home = match user.as_ref() {
        "root" => "/root".to_string(),
        _ => format!("/home/{}", user),
    };
    let dest_root = tilde_with_context(project.dest.as_str(), ||{
        if user == "root".to_string() {
            Some(Path::new("/root").into())
        } else {
            Some(Path::new(&common_home))
        }
    }).into_owned();
    info!("dest path: {}", dest_root);

    // get ignore dir
    let mut v = Vec::new();
    get_dir_ignored(&project.src, project.exclude.as_ref(), &mut v)?;

    //start watch
    let ignore_paths = if v.len() > 0 {Some(v)} else {None};
    start_watch(project.src, dest_root, &sftpclient, ignore_paths)?;

    Ok(())
}
