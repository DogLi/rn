use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;


pub fn load_file<T: AsRef<Path>>(file_path: T) -> io::Result<String> {

    let mut contents = String::new();
    File::open(file_path)?.read_to_string(&mut contents)?;
    Ok(contents)
}
