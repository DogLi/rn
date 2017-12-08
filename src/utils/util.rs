use errors::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;


/// get the content of a file
pub fn load_file(file_path: &Path) -> Result<String> {
    let mut contents = String::new();
    match File::open(file_path) {
        Ok(mut f) => {
            f.read_to_string(&mut contents).unwrap();
        },
        Err(e) => {
            let err_msg = format!("error when open file: {:?}, {:?}", file_path, e.to_string());
            bail!(err_msg.as_str());
        },
    }
    Ok(contents)
}

/// check if a path is exclude by regex
pub fn is_exclude(path: &Path, re_vec: &Vec<Regex>) -> bool{
    let coms = path.components();
    for com in coms {
        let com_str = com.as_os_str().to_str().unwrap();
        for re in re_vec.iter() {
            if re.is_match(com_str) {
                return true
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs::File;
    use std::io::prelude::*;
    use regex::Regex;

    #[test]
    fn test_load_file() {
        let tmp_path = Path::new("/tmp/test_load_file.txt");
        let mut tmp_file = File::create(&tmp_path).unwrap();
        let content = "hello world";
        tmp_file.write_all(content.as_bytes()).unwrap();
        let result = load_file(tmp_path).unwrap();
        assert_eq!(result, "hello world".to_string());
    }

    #[test]
    fn test_load_file2() {
        let path = Path::new("~/bin/setting.toml");
        let result = load_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_exclude() {
        let mut re_vec :Vec<Regex> = Vec::new();
        re_vec.push(Regex::new(r"\.git$").unwrap());
        let path1 = Path::new("a/b/.git");
        assert!(is_exclude(path1, &re_vec));
        let path2 = Path::new("a/b/.gitignore");
        assert!(!is_exclude(path2, &re_vec));
    }
}

