use errors::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;


pub fn load_file<T: AsRef<Path>>(file_path: T) -> Result<String> {
    println!("load file {:?}", file_path.as_ref());

    let mut contents = String::new();
    File::open(file_path.as_ref())
        .map_err(|err| format!("open {:?} failed: {}", file_path.as_ref(), err.to_string()))?
        .read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_load_file() {
        let tmp_path = Path::new("/tmp/test_load_file.txt");
        let mut tmp_file = File::create(&tmp_path).unwrap();
        let content = "hello world";
        tmp_file.write_all(content.as_bytes()).unwrap();
        let result = load_file(tmp_path).unwrap();
        assert_eq!(result, "hello world".to_string());
    }
}
