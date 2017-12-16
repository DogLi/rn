pub mod utils;
pub mod errors;
pub mod my_logger;
pub mod rsync;

extern crate glob;
extern crate ssh2;
extern crate serde;
extern crate notify;
extern crate regex;
extern crate toml;
extern crate shellexpand;


#[macro_use(slog_o, slog_debug, slog_info, slog_warn, slog_error, slog_crit, slog_log,
            slog_record, slog_record_static, slog_b, slog_kv)]
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
use std::sync::mpsc::channel;
use std::fs;
use regex::Regex;
use shellexpand::{tilde, tilde_with_context};


fn start_watch(
    src_path: &Path,
    dest_root: &Path,
    sftp: &ssh::SftpClient,
    exclude_files: &mut Vec<PathBuf>,
    include_files: &mut Vec<PathBuf>,
    re_vec: &Vec<Regex>,
) -> Result<()> {
    info!("watching path: {:?}", src_path);
    let (tx, rx) = channel();
    let mut watchdog = watchdog::WatchDog {
        src_path: src_path,
        dest_root: dest_root,
        tx: tx,
        rx: rx,
        sftp: sftp,
        exclude_files: exclude_files,
        include_files: include_files,
        re_vec: re_vec,
    };
    watchdog.start()?;
    Ok(())
}


fn get_file_ignored(
    root: &Path,
    re_vec: &Vec<Regex>,
    exclude_files: &mut Vec<PathBuf>,
    include_files: &mut Vec<PathBuf>,
) -> Result<()> {
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
                get_file_ignored(path_buf.as_path(), re_vec, exclude_files, include_files)?;
            }
        }
    }
    Ok(())
}

pub fn run(
    config_path: &Path,
    project_name: &str,
    server: &str,
    watch: bool,
    user: Option<&str>,
    password: Option<&str>,
    port: Option<u16>,
    identity: Option<&str>,
) -> Result<()> {
    let global_config = toml_parser::get_config(config_path)?;
    debug!("global config: {:?}", global_config);
    let project = toml_parser::get_project_info(project_name, &global_config)?;
    debug!("get project: {:?}", project);

    let ssh_conf_path = tilde("~/.ssh/config").into_owned();
    let server_host = sshconfig::parse_ssh_config(ssh_conf_path)?;
    debug!("server host: {:?}", server_host);
    let mut host: sshconfig::Host = match server_host.get(server) {
        Some(host) => {
            let mut host = host.clone();
            if host.identityfile.is_none() {
                host.password = global_config.global_password;
                if global_config.global_key.is_some() {
                    host.identityfile =
                        Some(Path::new(global_config.global_key.unwrap().as_str()).into());
                }
            }
            host
        }
        None => {
            //let hostname = sshconfig::get_ip();
            let hostname = sshconfig::get_ip(server)?;
            let g_user = global_config.global_user;
            // TODO: get password or key file from input
            let identityfile = match global_config.global_key {
                None => None,
                Some(ref file) => Some(tilde(file).into_owned()),
            };
            let g_password = global_config.global_password;
            let port = global_config.global_port;
            sshconfig::Host::new(hostname, g_user, identityfile, g_password, port)
        }
    };
    // update user, password, identity file
    if user.is_some() {
        host.user = user.unwrap().to_string();
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

    match port {
        Some(p) => {
            host.port = p;
        },
        None => {}
    }

    debug!("get host: {:?}, port {:?}", host, port);
    // TODO: use RC
    let tmp_host = host.clone();
    let sshclient = ssh::SSHClient::new(
        tmp_host.hostname,
        tmp_host.port,
        tmp_host.user,
        tmp_host.password,
        tmp_host.identityfile,
    )?;

    let sftpclient = ssh::SftpClient::new(&sshclient);

    // change ~ to /home/user or /root in dest path
    let common_home = match host.user.as_str() {
        "root" => "/root".to_string(),
        user => format!("/home/{}", user),
    };
    let dest_root = tilde_with_context(project.dest.as_str(), || if host.user.as_str() == "root" {
        Some(Path::new("/root").into())
    } else {
        Some(Path::new(&common_home))
    }).into_owned();
    let dest_str = dest_root.as_str();
    info!("dest path: {}", dest_str);
    let dest_path = Path::new(dest_str);

    // get ignore dir
    let mut exclude_files = Vec::new();
    let mut include_files = Vec::new();
    let mut re_vec: Vec<Regex> = Vec::new();
    match project.exclude {
        None => {}
        Some(ref vec) => {
            for v in vec.iter() {
                let re = util::create_re(v.as_str());
                if re.is_some() {
                    re_vec.push(re.unwrap());
                }
            }
        }
    }
    debug!("exclude setting: {:?}", re_vec);

    let src_path = Path::new(&project.src);
    // if src_path is link such as /tmp in MacOS, change it to real path, which is /private/tmp
    let real_path_buf = util::realpath(src_path)?;
    let src_path = real_path_buf.as_path();

    debug!("source file: {:?}", src_path);
    get_file_ignored(src_path, &re_vec, &mut exclude_files, &mut include_files)?;

    rsync::sync(
        host.identityfile,
        host.password,
        host.port,
        project.src.as_str(),
        dest_str,
        host.user.as_str(),
        host.hostname.as_str(),
        true,
        project.exclude,
    )?;

    //start watch
    start_watch(
        src_path,
        dest_path,
        &sftpclient,
        &mut exclude_files,
        &mut include_files,
        &re_vec,
    );

    Ok(())
}
