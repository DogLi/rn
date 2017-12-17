pub mod utils;
pub mod errors;
pub mod my_logger;

extern crate regex;
extern crate serde;
extern crate notify;
extern crate toml;
extern crate shellexpand;


#[macro_use]
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
use std::sync::mpsc::channel;
use shellexpand::{tilde, tilde_with_context};


fn start_watch(project: &toml_parser::Project, host: &sshconfig::Host) -> Result<()> {
    let (tx, rx) = channel();
    let mut watchdog = watchdog::WatchDog {
        project,
        host,
        tx: tx,
        rx: rx,
    };
    watchdog.start()?;
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
    let mut project = toml_parser::get_project_info(project_name, &global_config)?;
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
        }
        None => {}
    }

    match identity {
        Some(i) => {
            host.identityfile = Some(Path::new(i).to_path_buf());
            host.password = None;
        }
        None => {}
    }

    match port {
        Some(p) => {
            host.port = p;
        }
        None => {}
    }

    debug!("get host: {:?}, port {:?}", host, port);

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

    //    let src_path = Path::new(project.src);
    // if src_path is link such as /tmp in MacOS, change it to real path, which is /private/tmp
    let real_path_buf = util::realpath(Path::new(&project.src))?;
    let src_root = real_path_buf.into_os_string().into_string().unwrap();

    project.src = src_root;
    project.dest = dest_root;

    rsync::sync(&host, &project, true)?;

    //start watch
    if watch {
        start_watch(&project, &host)?;
    }

    Ok(())
}
