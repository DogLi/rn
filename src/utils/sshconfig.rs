use regex::Regex;
use errors::*;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io::{BufReader};
use std::fs::File;
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Debug;
use shellexpand::tilde;



#[derive(Debug, Clone)]
pub struct Host {
    pub HostName: String,
    pub IdentityFile: Option<PathBuf>,
    pub User: String,
    pub Password: Option<String>,
    pub Port: u16,
}


impl Host {
    pub fn new<S, P>(HostName: S, User: S, IdentityFile: Option<P>, Password: Option<S>, Port: Option<u16>) -> Self
    where S: AsRef<str>, P: AsRef<Path> {
        Host{
            HostName: HostName.as_ref().to_string(),
            User: User.as_ref().to_string(),
            IdentityFile: match IdentityFile {None => None, Some(ref file) => Some(file.as_ref().to_owned())},
            Password: match Password {None => None, Some(ref pwd) => Some(pwd.as_ref().to_string())},
            Port: Port.unwrap_or(22),
        }
    }
}

pub fn parse_ssh_config<P>(path: P) -> Result<HashMap<String, Host>>
    where P: AsRef<Path> + Debug{
    let mut result = HashMap::new();

    let f;
    let read_result = File::open(path.as_ref());
    match read_result {
        Ok(something) => f = something,
        Err(e) => {
            println!("open file {:?} failed: {:?}", path, e.to_string());
            return Ok(result);
        },
    }
    let mut file = BufReader::new(&f);

    let mut sections = Vec::new();
    let mut sub_section = Vec::new();

    for maybe_line in file.lines() {
        let line: String = try!(maybe_line);
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }
        if line.trim().starts_with("Host ") {
            if sub_section.len() > 0 {
                let _sub_section = sub_section.clone();
                sections.push(_sub_section);
                sub_section.clear();
                sub_section.push(line);
                continue;
            }
            else {
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
                host_obj.HostName = line_list[1].to_string();
            } else if line.to_lowercase().starts_with("user") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                host_obj.User = line_list[1].to_string();
            } else if line.to_lowercase().starts_with("identityfile") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                let key_path = tilde(line_list[1]).into_owned();
                host_obj.IdentityFile = Some(PathBuf::from(key_path));
            } else if line.to_lowercase().starts_with("Port") {
                let line_list: Vec<&str> = line.split_whitespace().collect();
                host_obj.Port = u16::from_str(line_list[1]).unwrap();
            }

        }
        result.insert(hostname, host_obj);
    }

    Ok(result)
}

/// q123 -> 192.168.1.123
/// 20 -> 10.10.20.20
/// 30.20 -> 10.10.30.20
pub fn get_ip<T: AsRef<str>>(hostname: T) -> String{
    let hostname = hostname.as_ref();
    let ip = if  Regex::new(r"^\d+$").unwrap().is_match(hostname) {
                format!("10.10.20.{}", hostname)
            } else if Regex::new(r"^\d+\.\d+$").unwrap().is_match(hostname) {
                format!("10.10.{}", hostname)
            } else if Regex::new(r"^q\d+$").unwrap().is_match(hostname) {
                format!("192.168.1.{}", hostname.split_at(1).1)
            } else {
                format!("{}", hostname)
            };
    ip
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_host() {
        let h = Host::new("10.10.90.11", "root", "~/.ssh/config");
        println!("{:?}", h);
    }

    #[test]
    #[ignore]
    fn test_parse_config() {
        let v = parse_ssh_config("/Users/yuanlinfeng/.ssh/config");
        println!("{:?}", v);
    }

    #[test]
    #[ignore]
    fn test_host_rule() {
        let host = get_host_by_rule("jiandong");
        println!("{:?}", host);
    }

    #[test]
    fn test_get_host_from_rule() {
        let host = get_host("jiandong2", "/Users/yuanlinfeng/.ssh/config");
        println!("test host: {:?}", host);
    }

    #[test]
    fn test_get_host_from_config() {
        let host = get_host("jiandong", "/Users/yuanlinfeng/.ssh/config");
        println!("test host: {:?}", host);
    }
}
