use regex::Regex;
use errors::*;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Debug;
use shellexpand::tilde;



#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub hostname: String,
    pub identityfile: Option<PathBuf>,
    pub user: String,
    pub password: Option<String>,
    pub port: u16,
}


impl Host {
    pub fn new<S, P>(
        hostname: S,
        user: S,
        identityfile: Option<P>,
        password: Option<S>,
        port: Option<u16>,
    ) -> Self
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        Host {
            hostname: hostname.as_ref().to_string(),
            user: user.as_ref().to_string(),
            identityfile: match identityfile {
                None => None,
                Some(ref file) => Some(file.as_ref().to_owned()),
            },
            password: match password {
                None => None,
                Some(ref pwd) => Some(pwd.as_ref().to_string()),
            },
            port: port.unwrap_or(22),
        }
    }
}

/// 解析ssh config文件
///
pub fn parse_ssh_config<P>(path: P) -> Result<HashMap<String, Host>>
where
    P: AsRef<Path> + Debug,
{
    let mut result = HashMap::new();

    let f = File::open(path.as_ref()).map_err(|err| {
        format!("open {:?} failed: {}", path, err.to_string())
    })?;
    let file = BufReader::new(&f);

    let mut sections = Vec::new();
    let mut sub_section = Vec::new();

    for maybe_line in file.lines() {
        let line: String = try!(maybe_line);
        if line.trim().len() == 0 || line.starts_with('#') {
            continue;
        }
        if line.trim().starts_with("Host ") {
            if sub_section.len() > 0 {
                let _sub_section = sub_section.clone();
                sections.push(_sub_section);
                sub_section.clear();
                sub_section.push(line);
                continue;
            } else {
                sub_section.push(line);
                continue;
            }
        } else {
            sub_section.push(line);
        }
    }
    // 最后的那一个主机配置
    if sub_section.len() > 0 {
        sections.push(sub_section);
    }

    for section in sections.iter() {
        // 虽然传入的是None, 但是需要将类型带上,否则会出现`cannot infer type for P`
        // 错误,参见: https://github.com/rust-lang/rust/issues/39797
        let mut host_obj: Host = Host::new("", "", None::<P>, None, None);
        let mut hostname = "".to_string();

        for line in section.iter() {
            let line_vec: Vec<&str> = line.trim().split("#").collect();
            let line = line_vec[0].trim();
            if line.to_lowercase().starts_with("host ") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                hostname = line_list[1].to_string();
            } else if line.to_lowercase().starts_with("hostname") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                host_obj.hostname = line_list[1].to_string();
            } else if line.to_lowercase().starts_with("user") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                host_obj.user = line_list[1].to_string();
            } else if line.to_lowercase().starts_with("identityfile") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                let key_path = tilde(line_list[1]).into_owned();
                host_obj.identityfile = Some(PathBuf::from(key_path));
            } else if line.to_lowercase().starts_with("Port") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                host_obj.port = u16::from_str(line_list[1])?;
            }

        }
        result.insert(hostname, host_obj);
    }

    Ok(result)
}

/// q123 -> 192.168.1.123
/// 20 -> 10.10.20.20
/// 30.20 -> 10.10.30.20
/// other_dns -> other_dns
pub fn get_ip<T: AsRef<str>>(hostname: T) -> Result<String> {
    let hostname = hostname.as_ref();
    let ip = if Regex::new(r"^\d+$")?.is_match(hostname) {
        format!("10.10.20.{}", hostname)
    } else if Regex::new(r"^\d+\.\d+$")?.is_match(hostname) {
        format!("10.10.{}", hostname)
    } else if Regex::new(r"^q\d+$")?.is_match(hostname) {
        format!("192.168.1.{}", hostname.split_at(1).1)
    } else {
        format!("{}", hostname)
    };
    Ok(ip)
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_parse_config() {
        let tmp_path = Path::new("/tmp/ssh_config");
        let mut tmp_file = File::create(&tmp_path).unwrap();
        let content = r##"Host pi
    HostName 10.10.80.83
    User pi
    PreferredAuthentications publickey
    IdentityFile ~/.ssh/id_rsa_foyu.pem

Host ubuntu
    HostName 192.168.75.129
    User ubuntu
    PreferredAuthentications publickey
    #IdentityFile ~/.ssh/id_rsa
"##;
        tmp_file.write_all(content.as_bytes()).unwrap();
        let v = parse_ssh_config(tmp_path).unwrap();

        let mut result = HashMap::new();
        result.insert(
            "pi".to_string(),
            Host {
                hostname: "10.10.80.83".to_string(),
                identityfile: Some(PathBuf::from(tilde("~/.ssh/id_rsa_foyu.pem").into_owned())),
                user: "pi".to_string(),
                password: None,
                port: 22,
            },
        );
        result.insert(
            "ubuntu".to_string(),
            Host {
                hostname: "192.168.75.129".to_string(),
                identityfile: None,
                user: "ubuntu".to_string(),
                password: None,
                port: 22,
            },
        );
        assert_eq!(result, v);
    }

    #[test]
    fn test_get_ip() {
        let ip = get_ip("baidu").unwrap();
        assert_eq!(ip, "baidu".to_string());
        let ip = get_ip("q11").unwrap();
        assert_eq!(ip, "192.168.1.11".to_string());
        let ip = get_ip("11").unwrap();
        assert_eq!(ip, "10.10.20.11".to_string());
        let ip = get_ip("30.11").unwrap();
        assert_eq!(ip, "10.10.30.11".to_string());
    }
}
