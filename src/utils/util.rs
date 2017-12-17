extern crate libc;

use errors::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use regex::Regex;


/// get the content of a file
pub fn load_file(file_path: &Path) -> Result<String> {
    let mut contents = String::new();
    match File::open(file_path) {
        Ok(mut f) => {
            f.read_to_string(&mut contents).unwrap();
        }
        Err(e) => {
            let err_msg = format!("error when open file: {:?}, {:?}", file_path, e.to_string());
            bail!(err_msg.as_str());
        }
    }
    Ok(contents)
}

/// check if a path is exclude by regex
pub fn is_exclude(path: &Path, re_vec: &Vec<Regex>) -> bool {
    let path_str = path.to_str().unwrap();
    println!("path to str: {:?}, re_vec: {:?}", path_str, re_vec);

    for re in re_vec.iter() {
        if re.is_match(path_str) {
            return true;
        }
    }
    false
}

/// create Regex from a given string,
/// the string is in as glob mode like *.jpg, a/*/*.jpg
pub fn create_re(normal_str: &str) -> Option<Regex> {
    let mut re_string = normal_str.to_string().clone();
    re_string = re_string.replace(".", r"\.").replace("*", r"[^/]*");
    re_string = format!(r"{}($|/)", re_string);
    if re_string.starts_with(r"/") {
        re_string = format!(r"^{}", re_string);
    }
    match Regex::new(re_string.as_str()) {
        Ok(re) => Some(re),
        Err(e) => {
            error!("error to create re: {}, {:?}", re_string, e);
            None
        }
    }
}

#[cfg(windows)]
pub fn realpath(original: &Path) -> io::Result<PathBuf> {
    Ok(original.to_path_buf())
}

#[cfg(unix)]
pub fn realpath(original: &Path) -> io::Result<PathBuf>{
    //use libc;
    info!("find real path for {:?}", original);
    use std::ffi::{OsString, CString};
    use std::os::unix::prelude::*;

    extern {
        fn realpath(pathname: *const libc::c_char, resolved: *mut libc::c_char)
                    -> *mut libc::c_char;
    }

    let path = CString::new(original.as_os_str().as_bytes())?;
    let mut buf = vec![0u8; 16 * 1024];
    unsafe {
        let r = realpath(path.as_ptr(), buf.as_mut_ptr() as *mut _);
        if r.is_null() {
            return Err(io::Error::last_os_error())
        }
    }
    let p = buf.iter().position(|i| *i == 0).unwrap();
    buf.truncate(p);
    Ok(PathBuf::from(OsString::from_vec(buf)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
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
        let mut re_vec: Vec<Regex> = Vec::new();
        re_vec.push(Regex::new(r"\.git$").unwrap());
        let path1 = Path::new("a/b/.git");
        assert!(is_exclude(path1, &re_vec));
        let path2 = Path::new("a/b/.gitignore");
        assert!(!is_exclude(path2, &re_vec));
    }

    #[test]
    fn test_create_re_string() {
        let a = "*.jpg";
        let some_re = create_re(a);
        assert!(some_re.is_some());
        let re = &some_re.unwrap();
        assert!(re.is_match("a/b/c.jpg"));
        assert!(!re.is_match("a/b/c.jpga"));

        let a = "a/b/*.jpg";
        let some_re = create_re(a);
        assert!(some_re.is_some());
        let re = &some_re.unwrap();
        assert!(re.is_match("a/b/c.jpg"));
        assert!(!re.is_match("a/c.jpg"));
        assert!(!re.is_match("a/b/c.jpga"));
        assert!(!re.is_match("a/b1/c.jpg"));

        let a = "a/*/*.jpg";
        let some_re = create_re(a);
        assert!(some_re.is_some());
        let re = &some_re.unwrap();
        assert!(re.is_match("a/b/c.jpg"));
        assert!(re.is_match("a/b1/c.jpg"));
        assert!(!re.is_match("a/b/c.jpga"));
        assert!(!re.is_match("a/c.jpg"));

        let a = "/a/b/*.jpg";
        let some_re = create_re(a);
        assert!(some_re.is_some());
        let re = &some_re.unwrap();
        assert!(re.is_match("/a/b/c.jpg"));
        assert!(!re.is_match("/a/a/b/c.jpg"));

        let a = "hello*world";
        let some_re = create_re(a);
        assert!(some_re.is_some());
        let re = &some_re.unwrap();
        assert!(re.is_match("a/helloabcworld/b"));
        assert!(re.is_match("helloworld"));
    }

    #[test]
    fn test_real_path() {
        let path = Path::new("/tmp/a");
        match realpath(path) {
            Ok(path_buf) => {
                assert_eq!(path_buf, PathBuf::from("/private/tmp/a"));
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
