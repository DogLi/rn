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
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::sync::mpsc:: channel;
use std::fs;
use regex::Regex;
use shellexpand::{tilde, tilde_with_context};
use notify::DebouncedEvent;
use std::os::unix::fs::PermissionsExt;


fn start_watch(src_path: &Path, dest_root: &Path, sftp: &ssh::SftpClient, exclude_files: &mut Vec<PathBuf>, mut include_files: &mut Vec<PathBuf>, re_vec: &Vec<Regex>) -> Result<()>{
    info!("watching path: {:?}", src_path);
    let (tx, rx) = channel();
    let mut watchdog = watchdog::WatchDog {
        src_path: src_path,
        dest_root: dest_root,
        tx: tx,
        rx:rx,
        sftp: sftp,
        exclude_files: exclude_files,
        include_files: include_files,
        re_vec: re_vec,
    };
    watchdog.start()?;
    Ok(())
}


fn get_file_ignored(root: &Path, re_vec: &Vec<Regex>, exclude_files: &mut Vec<PathBuf>, include_files: &mut Vec<PathBuf>) -> Result<()> {
    if util::is_exclude(&root, re_vec) {
        info!("find unignored path: {:?}", root.as_os_str());
        exclude_files.push(root.to_path_buf())
    } else {
        include_files.push(root.to_path_buf())
    }
    if !root.metadata()?.file_type().is_dir() {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path_buf = entry.path();
            if util::is_exclude(path_buf.as_path(), re_vec) {
                exclude_files.push(path_buf.clone());
            } else {
                include_files.push(path_buf.clone());
            }
            if path_buf.is_dir() {
                get_file_ignored(path_buf.as_path(), re_vec, exclude_files, include_files);
            }
        }
    }
    Ok(())
}

pub fn run(config_path: &Path, project_name: &str, server: &str, watch: bool, user: Option<&str>, password: Option<&str>, identity: Option<&str>) -> Result<()> {
    let log = slog_scope::logger();
    // get the global config
    let global_config = toml_parser::get_config(config_path)?;
    info!("global config: {:?}", global_config);
    let project = toml_parser::get_project_info(project_name, &global_config)?;
    info!("get project: {:?}", project);

    // get host config
    let ssh_conf_path = tilde("~/.ssh/config").into_owned();

    let server_host = sshconfig::parse_ssh_config(ssh_conf_path)?;
    let mut host: sshconfig::Host = match server_host.get(server) {
        Some(host) => {
            let mut host = host.clone();
            if host.identityfile.is_none() {
                host.password = global_config.global_password;
                if global_config.global_key.is_some() {
                    host.identityfile = Some(Path::new(global_config.global_key.unwrap().as_str()).into());
                }
            }
            host
        },
        None => {
            //let hostname = sshconfig::get_ip();
            let hostname = sshconfig::get_ip(server)?;
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
    match user {
        Some(u) => {
            host.user = u.to_string();
        },
        None => {}
    }

    match password {
        Some(p) => {
            host.password = Some(p.to_string());
            host.identityfile = None;
        },
        None => {}
    }

    match identity {
        Some(i) => {
            host.identityfile = Some(Path::new(i).to_path_buf());
            host.password = None;
        },
        None => {}
    }

    info!("get host: {:?}", host);

    // connect
    let user = host.user.clone();
    let sshclient = ssh::SSHClient::new(host.hostname, host.port, host.user, host.password, host.identityfile)?;
    let cmd_output = sshclient.run_cmd("whoami")?;
    info!("get cmd 'whoami' result: {:?}", cmd_output);
    let sftpclient = ssh::SftpClient::new(&sshclient);

    // change ~ to /home/user or /root in dest path
    let common_home = match user.as_str() {
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
    let dest_path = Path::new(dest_root.as_str());

    // get ignore dir
    let mut exclude_files = Vec::new();
    let mut include_files = Vec::new();
    let mut re_vec :Vec<Regex> = Vec::new();
    match project.exclude {
        None =>{},
        Some(vec) => {
            for v in vec.iter() {
                let tmp_re = Regex::new(v).unwrap();
                re_vec.push(tmp_re);
            }
        }
    }
    info!("exclude setting: {:?}", re_vec);

    let src_path = Path::new(&project.src);
    get_file_ignored(src_path, &re_vec, &mut exclude_files, &mut include_files)?;

    //start watch
    start_watch(src_path, dest_path, &sftpclient, &mut exclude_files, &mut include_files, &re_vec)?;

    Ok(())
}
