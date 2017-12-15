use errors::*;
use std::process::Command;

pub fn rsync(identity: &str,
             source: &str,
             dest: &str,
             username: &str,
             remote_ip: &str,
             delete: bool,
             exclude_files: Option<Vec<&str>>) -> Result<()>{
    let login_settings = format!(r#"ssh -i {} -o "UserKnownHostsFile=/dev/null" -o "StrictHostKeyChecking no" -o "ConnectTimeout=2""#, identity);
    let mut cmd = Command::new("rsync");
    cmd.arg("-rtv").arg("-e").arg(login_settings);
    if delete {
        cmd.arg("--delete");
    }
    match exclude_files {
        None => {},
        Some(exclude_files) => {
            for file in exclude_files.iter(){
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

    #[test]
    fn test_rsync() {
        let id = tilde("~/.ssh/id_rsa").into_owned();
        let id = id.as_str();
        let source = "/tmp/a";
        let dest = "/home/ubuntu/a";
        let remote_ip = "ubuntu";
        let username = "ubuntu";
        let exclude_files = vec!["a.txt", "b.txt", "c.txt"];
        if let Err(e) = rsync(id, source, dest, username, remote_ip, false, Some(exclude_files)) {
            assert!(false, "aaaaaaaaaaaaa");
        } else {
            assert!(true);
        }
    }
}