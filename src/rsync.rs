use std::io;
use std::path::Path;
use std::process::Command;

pub fn rsync<P, S>(
    identity: Option<P>,
    password: Option<S>,
    port: u16,
    source: &str,
    dest: &str,
    username: &str,
    remote_ip: &str,
    delete: bool,
    exclude_files: Option<&Vec<&str>>,
) -> io::Result<()>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let login_strings: String;
    match identity {
        None => {
            match password {
                None => {
                    panic!("no password or identifile was set!");
                }
                Some(password) => {
                    let pwd = password.as_ref();
                    login_strings = format!(r#"sshpass {} ssh  -l {} -p {}"#, pwd, username, port);
                }
            }
        }
        Some(path) => {
            let path = path.as_ref().to_str().unwrap();
            login_strings = format!(r#"ssh -i {} -p {} -o "UserKnownHostsFile=/dev/null" \
                                                       -o "StrictHostKeyChecking no" \
                                                       -o "ConnectTimeout=2""#, path, port);
        }
    }
    let mut cmd = Command::new("rsync");
    cmd.arg("-rtv").arg("-e").arg(login_strings);
    if delete {
        cmd.arg("--delete");
    }
    match exclude_files {
        None => {}
        Some(exclude_files) => {
            for file in exclude_files.iter() {
                cmd.arg("--exclude").arg(file);
            }
        }
    }
    let target = format!("{}@{}:{}", username, remote_ip, dest);
    cmd.arg(source).arg(target);
    let output = cmd.output()?;
    if output.stdout.len() > 0 {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        info!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
    if output.stderr.len() > 0 {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use shellexpand::tilde;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_rsync() {
        let source = "/tmp/a";
        let dest = "/home/ubuntu/a";
        let remote_ip = "ubuntu"; // 10.10.20.3 in /etc/hosts file
        let username = "ubuntu";
        let port = 2222;
        let exclude_files = vec!["a.txt", "b.txt", "c.txt"];

        // test use identity file
        let id = tilde("~/.ssh/id_rsa.bak").into_owned();
        let id_file = Path::new(id.as_str());
        let password = None::<&str>;
        if let Err(e) = rsync(
            Some(id_file),
            password,
            port,
            source,
            dest,
            username,
            remote_ip,
            false,
            Some(&exclude_files),
        )
        {
            assert!(false, "rsync test indentity file failed");
        } else {
            assert!(true);
        }

        // test use password
        let id = None::<PathBuf>;
        let password = Some("nogame");
        if let Err(e) = rsync(
            Some(id_file),
            password,
            port,
            source,
            dest,
            username,
            remote_ip,
            false,
            Some(&exclude_files),
        )
        {
            assert!(false, "rsync test password failed");
        } else {
            assert!(true);
        }
    }
}
