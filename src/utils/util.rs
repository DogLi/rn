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
    let path_str = path.to_str().unwrap();
    println!("path to str: {:?}, re_vec: {:?}", path_str, re_vec);

    for re in re_vec.iter() {
        if re.is_match(path_str) {
            return true
        }
    }
    false
}

pub fn create_re(normal_str: &str) -> Option<Regex> {
    let mut re_string = normal_str.to_string().clone();
    re_string = re_string
        .replace(".", r"\.")
        .replace("*", r"[^/]*");
    re_string = format!(r"{}[$/]", re_string);
    if !re_string.starts_with(r"/") {
        re_string = format!(r"^{}", re_string);
    }
    println!("{}", re_string);
    match Regex::new(re_string.as_str()) {
        Ok(re) => { Some(re) },
        Err(e) => {
            error!("error to create re: {}, {:?}", re_string, e);
            None
        }
    }
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

    #[test]
    fn test_create_re_string() {
        let a = "*.jpg";
        let re =  create_re(a);
        let re = Some(Regex::new(r"^[^/]*\.jpg[$/]").unwrap());
        assert!(re.is_some());
        match re{
            Some(re) => {
                println!("re: {:?}", re);
                assert!(re.is_match(r"a/b/c.jpg"));
                assert!(!re.is_match(r"a/b/c.jpga"));
            },
            None => {}
        }




//        let a = "/tmp/*.jpg";
//        let re = create_re(a).unwrap();
//        let path1 = Path::new("/tmp/a.jpg");
//        let path2 = Path::new("/tmp/a/a.jpg");
//        let re_vec = vec!(re);
//        assert!(is_exclude(path1, &re_vec));
//        assert!(!is_exclude(path2, &re_vec));
    }
}

