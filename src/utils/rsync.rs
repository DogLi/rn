use std::{io, fs};
use std::path::Path;
use std::process::Command;
use super::sshconfig::servername2ip;
use super::sshconfig::Host;
use super::toml_parser::Project;

pub fn sync(host: &Host, project: &Project, delete: bool) -> io::Result<()> {
    let username = &host.user;
    let path = Path::new(project.src.as_str());
    let file_type = fs::metadata(path)?.file_type();
    // if the source file is directory and not ends with "/", we should add it.
    let mut source = String::from(project.src.as_str());
    if file_type.is_dir() && !source.ends_with("/") {
        source.push_str("/")
    }
    debug!("source file is {:?}", source);
    let remote_ip = servername2ip(host.hostname.as_str());

    let login_strings: String;
    match host.identityfile {
        None => {
            match host.password {
                None => {
                    panic!("no password or identifile was set!");
                }
                Some(ref password) => {
                    login_strings = format!(
                        r#"sshpass -p {} ssh  -l {} -p {} -o "ConnectTimeout=2""#,
                        password,
                        username,
                        host.port
                    );
                }
            }
        }
        Some(ref path) => {
            let path = path.to_str().unwrap();
            login_strings = format!(r#"ssh -i {} -p {} -o "UserKnownHostsFile=/dev/null" -o "StrictHostKeyChecking no" -o "ConnectTimeout=2""#,
                path,
                host.port
            );
        }
    }
    let mut cmd = Command::new("rsync");
    cmd.arg("-rtv").arg("-e").arg(login_strings);
    if delete {
        cmd.arg("--delete");
    }
    match project.exclude {
        None => {}
        Some(ref exclude_files) => {
            for file in exclude_files.iter() {
                cmd.arg("--exclude").arg(file);
            }
        }
    }
    let target = format!("{}@{}:{}", username, remote_ip, project.dest);
    cmd.arg(source).arg(target);
    debug!("{:?}", cmd);
    let output = cmd.output()?;
    if output.stdout.len() > 0 {
        info!("rsync output:\n {}", String::from_utf8_lossy(&output.stdout));
    }
    if output.stderr.len() > 0 {
        error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::sshconfig::Host;
    use super::super::toml_parser::Project;
    use shellexpand::tilde;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_sync() {
        let id = tilde("~/.ssh/id_rsa").into_owned();
        let host = Host::new("ubuntu", "ubuntu", Some(PathBuf::from(id)), None, Some(2222));
        let project = Project {
            name: "test".to_string(),
            src: "/tmp/a".to_string(),
            dest: "/home/ubuntu/a".to_string(),
            exclude: Some(vec![
                "a.txt".to_string(),
                "b.txt".to_string(),
            ]),
        };

        if let Err(e) = sync(&host, &project, true) {
            assert!(false, "rsync test password failed");
        } else {
            assert!(true);
        }

        let host = Host::new("ubuntu", "ubuntu", None::<PathBuf>, Some("ubuntu"), Some(2222));
        let project = Project {
            name: "test".to_string(),
            src: "/tmp/b".to_string(),
            dest: "/home/ubuntu/b".to_string(),
            exclude: Some(vec![
                "a.txt".to_string(),
                "b.txt".to_string(),
            ]),
        };
        if let Err(e) = sync(&host, &project, true) {
            assert!(false, "rsync test password failed");
        } else {
            assert!(true);
        }
    }
}
