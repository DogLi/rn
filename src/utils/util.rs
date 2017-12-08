use errors::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;


/// get the content of a file
pub fn load_file<T: AsRef<Path>>(file_path: T) -> Result<String> {
    println!("load file {:?}", file_path.as_ref());

    let mut contents = String::new();
    File::open(file_path.as_ref())
        .map_err(|err| format!("open {:?} failed: {}", file_path.as_ref(), err.to_string()))?
        .read_to_string(&mut contents)?;
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
    fn test_is_exclude() {
        let mut re_vec :Vec<Regex> = Vec::new();
        re_vec.push(Regex::new(r"\.git$").unwrap());
        let path1 = Path::new("a/b/.git");
        assert!(is_exclude(path1, &re_vec));
        let path2 = Path::new("a/b/.gitignore");
        assert!(!is_exclude(path2, &re_vec));
    }
}

